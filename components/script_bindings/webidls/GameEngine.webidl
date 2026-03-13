[Exposed=Window]
interface GameEngine {
    [Throws] constructor();
    boolean spawnEnemy(DOMString enemy_id, unrestricted float x, unrestricted float y);
};
