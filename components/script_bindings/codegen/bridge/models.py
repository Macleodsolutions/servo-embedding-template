"""
Data model layer for the Embedder Bridge codegen.

The IDL AST is walked once and converted into plain Python dataclasses.
All generation code works exclusively against these objects - no AST access
outside this module.
"""
from __future__ import annotations

from dataclasses import dataclass
import re


# ---------------------------------------------------------------------------
# IDL type -> Rust type
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class BridgeType:
    kind: str
    inner: BridgeType | None = None
    name: str | None = None

    @staticmethod
    def from_idl(idl_type) -> BridgeType:
        if idl_type.nullable():
            return BridgeType("option", inner=BridgeType.from_idl(idl_type.inner))
        if idl_type.isSequence():
            return BridgeType("sequence", inner=BridgeType.from_idl(idl_type.inner))
        if idl_type.isDOMString() or idl_type.isUSVString() or idl_type.isByteString():
            return BridgeType("string")
        if idl_type.isBoolean():
            return BridgeType("bool")
        if idl_type.isFloat():
            from WebIDL import IDLBuiltinType

            if idl_type.tag() in (
                IDLBuiltinType.Types.float,
                IDLBuiltinType.Types.unrestricted_float,
            ):
                return BridgeType("builtin", name="f32")
            return BridgeType("builtin", name="f64")
        if idl_type.isInteger():
            from WebIDL import IDLBuiltinType

            return BridgeType(
                "builtin",
                name={
                    IDLBuiltinType.Types.byte: "i8",
                    IDLBuiltinType.Types.octet: "u8",
                    IDLBuiltinType.Types.short: "i16",
                    IDLBuiltinType.Types.unsigned_short: "u16",
                    IDLBuiltinType.Types.long: "i32",
                    IDLBuiltinType.Types.unsigned_long: "u32",
                    IDLBuiltinType.Types.long_long: "i64",
                    IDLBuiltinType.Types.unsigned_long_long: "u64",
                }.get(idl_type.tag(), "i32"),
            )
        return BridgeType("other", name=str(idl_type))

    def to_rust(self) -> str:
        if self.kind == "option":
            return f"Option<{self.inner.to_rust()}>"
        if self.kind == "sequence":
            return f"Vec<{self.inner.to_rust()}>"
        if self.kind == "string":
            return "String"
        if self.kind == "bool":
            return "bool"
        if self.kind in ("builtin", "other"):
            return self.name
        raise ValueError(f"Unsupported BridgeType kind: {self.kind}")

    def to_webidl(self) -> str:
        if self.kind == "option":
            return f"{self.inner.to_webidl()}?"
        if self.kind == "sequence":
            return f"sequence<{self.inner.to_webidl()}>"
        if self.kind == "string":
            return "DOMString"
        if self.kind == "bool":
            return "boolean"
        if self.kind == "builtin":
            return {
                "f32": "unrestricted float",
                "f64": "unrestricted double",
                "i8": "byte",
                "u8": "octet",
                "i16": "short",
                "u16": "unsigned short",
                "i32": "long",
                "u32": "unsigned long",
                "i64": "long long",
                "u64": "unsigned long long",
            }.get(self.name, self.name)
        if self.kind == "other":
            return self.name
        raise ValueError(f"Unsupported BridgeType kind: {self.kind}")

    def to_binding_type(self) -> str:
        if self.kind == "option":
            return f"Option<{self.inner.to_binding_type()}>"
        if self.kind == "sequence":
            return f"Vec<{self.inner.to_binding_type()}>"
        if self.kind == "string":
            return "DOMString"
        return self.to_rust()

    def _to_dom_expr(self, var_name: str) -> str:
        if self.kind == "string":
            return f"DOMString::from({var_name})"
        if self.kind == "option":
            inner_expr = self.inner._to_dom_expr("v")
            if inner_expr == "v":
                return var_name
            return f"{var_name}.map(|v| {inner_expr})"
        if self.kind == "sequence":
            inner_expr = self.inner._to_dom_expr("v")
            if inner_expr == "v":
                return var_name
            return f"{var_name}.into_iter().map(|v| {inner_expr}).collect()"
        return var_name

    def _from_dom_expr(self, var_name: str) -> str:
        if self.kind == "string":
            return f"String::from({var_name})"
        if self.kind == "option":
            inner_expr = self.inner._from_dom_expr("v")
            if inner_expr == "v":
                return var_name
            return f"{var_name}.map(|v| {inner_expr})"
        if self.kind == "sequence":
            inner_expr = self.inner._from_dom_expr("v")
            if inner_expr == "v":
                return var_name
            return f"{var_name}.into_iter().map(|v| {inner_expr}).collect()"
        return var_name


def idl_type_to_rust(idl_type) -> str:
    return BridgeType.from_idl(idl_type).to_rust()


def idl_type_to_bridge_type(idl_type) -> BridgeType:
    return BridgeType.from_idl(idl_type)


def rust_type_to_webidl(rust_type: str) -> str:
    if rust_type.startswith("Option<"):
        return f"{rust_type_to_webidl(rust_type[7:-1])}?"
    if rust_type.startswith("Vec<"):
        return f"sequence<{rust_type_to_webidl(rust_type[4:-1])}>"
    return {
        "String": "DOMString",
        "bool": "boolean",
        "f32": "unrestricted float",
        "f64": "unrestricted double",
        "i8": "byte",
        "u8": "octet",
        "i16": "short",
        "u16": "unsigned short",
        "i32": "long",
        "u32": "unsigned long",
        "i64": "long long",
        "u64": "unsigned long long",
    }.get(rust_type, rust_type)


# ---------------------------------------------------------------------------
# Name helpers
# ---------------------------------------------------------------------------

def pascal_case(name: str) -> str:
    parts = re.sub(r"([A-Z])", r"_\1", name).split("_")
    return "".join(p.capitalize() for p in parts if p)


def to_snake_case(name: str) -> str:
    return re.sub(r"([A-Z])", lambda m: "_" + m.group(1).lower(), name).lstrip("_")


# ---------------------------------------------------------------------------
# Dataclasses
# ---------------------------------------------------------------------------

@dataclass
class BridgeField:
    name: str
    type_: BridgeType

    @property
    def rust_type(self) -> str:
        return self.type_.to_rust()

    @property
    def webidl_type(self) -> str:
        return self.type_.to_webidl()

    @property
    def is_string(self) -> bool:
        return self.type_.kind == "string"

    @property
    def is_vec_string(self) -> bool:
        return self.type_.kind == "sequence" and self.type_.inner.kind == "string"

    @property
    def is_float(self) -> bool:
        return self.rust_type in ("f32", "f64")

    @property
    def dom_type(self) -> str:
        """Type used in DOM struct fields (DOMString instead of String)."""
        return self.type_.to_binding_type()

    @property
    def as_dom_expr(self) -> str:
        """Expression to convert a local variable of rust_type to dom_type."""
        return self.type_._to_dom_expr(self.name)


@dataclass
class BridgeArg:
    name: str
    type_: BridgeType

    @property
    def rust_type(self) -> str:
        return self.type_.to_rust()

    @property
    def webidl_type(self) -> str:
        return self.type_.to_webidl()

    @property
    def is_string(self) -> bool:
        return self.type_.kind == "string"

    @property
    def is_vec_string(self) -> bool:
        return self.type_.kind == "sequence" and self.type_.inner.kind == "string"

    @property
    def trait_type(self) -> str:
        """Type used in the generated WebIDL binding trait (DOMString, not String)."""
        return self.type_.to_binding_type()

    @property
    def as_string_expr(self) -> str:
        """Expression to convert binding arg type to the EmbedderMsg rust_type."""
        return self.type_._from_dom_expr(self.name)


@dataclass
class BridgeMethod:
    name: str           # camelCase IDL name, e.g. "spawnEnemy"
    iface_name: str
    args: list[BridgeArg]

    @property
    def variant_name(self) -> str:
        return self.iface_name + pascal_case(self.name)

    @property
    def rust_name(self) -> str:
        return pascal_case(self.name)

    @property
    def delegate_method(self) -> str:
        return "handle_" + to_snake_case(self.variant_name)


@dataclass
class BridgeEvent:
    attr_name: str      # e.g. "onenemydied"
    iface_name: str
    fields: list[BridgeField]

    @property
    def event_name(self) -> str:
        """PascalCase event name, e.g. 'EnemyDied'"""
        return pascal_case(self.attr_name[2:])

    @property
    def event_atom(self) -> str:
        """Lowercase DOM atom, e.g. 'enemydied'"""
        return self.event_name.lower()

    @property
    def event_iface(self) -> str:
        """e.g. 'GameEngineEnemyDiedEvent'"""
        return f"{self.iface_name}{self.event_name}Event"

    @property
    def event_iface_lower(self) -> str:
        return self.event_iface.lower()

    @property
    def variant_name(self) -> str:
        return f"{self.iface_name}{self.event_name}"

    @property
    def fire_method(self) -> str:
        return f"fire_{self.event_atom}"

    @property
    def webview_fire_method(self) -> str:
        return f"fire_{self.iface_name.lower()}_{self.event_atom}"

    @property
    def getter(self) -> str:
        return f"Get{pascal_case(self.attr_name)}"

    @property
    def setter(self) -> str:
        return f"Set{pascal_case(self.attr_name)}"


@dataclass
class BridgeInterface:
    name: str
    methods: list[BridgeMethod]
    events: list[BridgeEvent]

    @property
    def name_lower(self) -> str:
        return self.name.lower()

    @property
    def name_camel(self) -> str:
        """First-letter-lowercased, e.g. 'gameEngine'"""
        return self.name[0].lower() + self.name[1:]


# ---------------------------------------------------------------------------
# AST -> model
# ---------------------------------------------------------------------------

_EVENTINIT_FIELDS = {"bubbles", "cancelable", "composed"}


def _find_event_init_dict(parser_results, event_name: str, iface_name: str):
    candidates = {f"{iface_name}{event_name}EventInit", f"{event_name}EventInit"}
    for thing in parser_results:
        if hasattr(thing, "members") and hasattr(thing, "identifier"):
            if thing.identifier.name in candidates:
                return thing
    return None


def _dict_fields(dictionary) -> list[BridgeField]:
    return [
        BridgeField(m.identifier.name, idl_type_to_bridge_type(m.type))
        for m in dictionary.members
        if m.identifier.name not in _EVENTINIT_FIELDS
    ]


def extract_interfaces(parser_results) -> list[BridgeInterface]:
    interfaces = []
    for thing in parser_results:
        if not hasattr(thing, "members") or not hasattr(thing, "getExtendedAttribute"):
            continue
        if not thing.getExtendedAttribute("EmbedderBridge"):
            continue

        iface_name = thing.identifier.name
        methods: list[BridgeMethod] = []
        events: list[BridgeEvent] = []

        for member in thing.members:
            if member.isMethod() and member.identifier.name != "constructor":
                _, args = member.signatures()[0]
                methods.append(
                    BridgeMethod(
                        name=member.identifier.name,
                        iface_name=iface_name,
                        args=[
                            BridgeArg(a.identifier.name, idl_type_to_bridge_type(a.type))
                            for a in args
                        ],
                    )
                )
            elif member.isAttr():
                attr_name = member.identifier.name
                if not attr_name.startswith("on"):
                    continue
                if not str(member.type).startswith("EventHandler"):
                    continue
                event_name = pascal_case(attr_name[2:])
                dictionary = _find_event_init_dict(parser_results, event_name, iface_name)
                events.append(
                    BridgeEvent(
                        attr_name=attr_name,
                        iface_name=iface_name,
                        fields=_dict_fields(dictionary) if dictionary else [],
                    )
                )

        interfaces.append(BridgeInterface(name=iface_name, methods=methods, events=events))

    return interfaces
