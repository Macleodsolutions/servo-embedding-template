const fs = require('fs');
const webidl2 = require('webidl2');

const idlText = fs.readFileSync('../../../components/script_bindings/webidls/GameEngine.webidl', 'utf8');
const ast = webidl2.parse(idlText);

let dts = '';

function mapType(idlType) {
    const typeName = idlType.idlType;
    switch (typeName) {
        case 'DOMString': return 'string';
        case 'float':
        case 'double':
        case 'short':
        case 'long':
        case 'long long':
        case 'unsigned short':
        case 'unsigned long':
        case 'unrestricted float':
        case 'unrestricted double':
            return 'number';
        case 'boolean': return 'boolean';
        default: return 'any';
    }
}

for (const node of ast) {
    if (node.type === 'interface') {
        dts += `interface ${node.name} {\n`;
        for (const member of node.members) {
            if (member.type === 'operation') {
                if (!member.name) continue;
                const args = member.arguments.map(arg => `${arg.name}: ${mapType(arg.idlType)}`).join(', ');
                const ret = member.idlType ? mapType(member.idlType) : 'void';
                dts += `    ${member.name}(${args}): ${ret};\n`;
            }
        }
        dts += `}\n\n`;
    }
}

dts += `interface Window {\n    gameEngine: GameEngine;\n}\n`;

fs.writeFileSync('GameEngine.d.ts', dts);
console.log('Generated GameEngine.d.ts successfully.');
console.log(dts);
