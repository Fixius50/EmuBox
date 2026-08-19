# Reglas de Arquitectura y Refactorización para EmuBox

1. **Separación Visual vs Lógica**:
   - Componentes (`.tsx`): Exclusivamente presentación pura.
   - Lógica de negocio, efectos y estados: Exclusivamente en Custom Hooks (`solid/src/hooks/use*.ts`).
2. **Interfaces Centralizadas**:
   - Prohibido declarar interfaces complejas inline en componentes o hooks.
   - Todas las interfaces y contratos deben residir en `solid/src/types/`.
3. **Componentes Comunes**:
   - Centralizar y reutilizar componentes atómicos en `solid/src/components/common/` (`Badge`, `ConsoleButton`, `SettingCardRow`).
4. **Control de Flujo Interno**:
   - Priorizar ternarios y declaraciones `switch` sobre cascadas de `if/else if`.
   - Prohibido `switch (true)` y cláusulas `default:` vacías sin propósito explícito.
5. **Invariante Visual y de Consola**:
   - Prohibido el uso de emojis en cualquier archivo o UI de la aplicación.
   - Navegación 100% compatible con mando físico y teclado (10-foot UI).
