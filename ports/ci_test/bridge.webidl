/*
 * Minimal stubs to satisfy the WebIDL parser's type resolution.
 * Not used for code generation.
 */
interface Event {};

[LegacyTreatNonObjectAsNull]
callback EventHandlerNonNull = any (Event event);
typedef EventHandlerNonNull? EventHandler;

interface EventTarget {
    [Throws] constructor();
};

dictionary EventInit {
    boolean bubbles = false;
    boolean cancelable = false;
    boolean composed = false;
};

/*
 * GameEngine bridge interface.
 *
 * Methods  → JS calls Rust → EmbedderMsg → WebViewDelegate
 * Events   → Rust fires JS event via WebView::fire_gameengine_*
 */
[EmbedderBridge]
interface GameEngine : EventTarget {
    /* JS → Rust: request the embedder to spawn an enemy */
    boolean spawnEnemy(DOMString enemyId, unrestricted float x, unrestricted float y);

    /* Rust → JS: fire when an enemy has died */
    attribute EventHandler onenemydied;
};

dictionary GameEngineEnemydiedEventInit : EventInit {
    required DOMString enemyId;
    required unrestricted float x;
    required unrestricted float y;
};
