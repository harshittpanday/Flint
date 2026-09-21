package dev.flint.client;

import java.util.List;

public final class ModuleRegistry {
    private static final List<ModuleDefinition> MODULES = List.of(
            new ModuleDefinition("fps", "FPS", "Current frame rate", ModuleDefinition.Category.HUD),
            new ModuleDefinition("coordinates", "Coordinates", "Current world position", ModuleDefinition.Category.HUD),
            new ModuleDefinition("ping", "Ping", "Connection latency", ModuleDefinition.Category.HUD),
            new ModuleDefinition("speed", "Speed", "Horizontal movement speed", ModuleDefinition.Category.HUD),
            new ModuleDefinition("memory", "Memory", "Java heap usage", ModuleDefinition.Category.HUD),
            new ModuleDefinition("keystrokes", "Keystrokes", "Movement and mouse input", ModuleDefinition.Category.HUD),
            new ModuleDefinition("reach", "Reach display", "Distance to the current target", ModuleDefinition.Category.HUD),
            new ModuleDefinition("armor", "Armor status", "Equipped armor durability", ModuleDefinition.Category.HUD),
            new ModuleDefinition("fullbright", "Fullbright", "Client-side visibility boost", ModuleDefinition.Category.RENDER)
    );

    private ModuleRegistry() {}

    public static List<ModuleDefinition> all() {
        return MODULES;
    }
}
