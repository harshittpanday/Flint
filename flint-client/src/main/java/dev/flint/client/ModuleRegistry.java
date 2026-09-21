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
            new ModuleDefinition("effects", "Effects", "Active status effects", ModuleDefinition.Category.HUD),
            new ModuleDefinition("cps", "CPS", "Left and right clicks per second", ModuleDefinition.Category.HUD),
            new ModuleDefinition("combo", "Combo", "Consecutive entity attacks", ModuleDefinition.Category.HUD),
            new ModuleDefinition("fullbright", "Fullbright", "Client-side visibility boost", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("zoom", "Zoom", "Hold C for a focused view", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("freelook", "Freelook", "Hold Left Alt to look independently", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("time_changer", "Time Changer", "Local fixed daytime view", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("weather_changer", "Weather Changer", "Local clear-weather view", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("custom_hand", "Custom Hand", "Compact first-person hand position", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("projectile_trails", "Projectile Trails", "Local amber projectile particles", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("player_particles", "Player Particles", "Local movement particles", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("audio_visualizer", "Audio Visualizer", "Local bars driven by game sound events", ModuleDefinition.Category.RENDER),
            new ModuleDefinition("toggle_sprint", "Toggle Sprint", "Tap Sprint to latch while moving", ModuleDefinition.Category.PLAYER),
            new ModuleDefinition("auto_gg", "Auto GG", "One rate-limited GG after clear victory text", ModuleDefinition.Category.PLAYER)
    );

    private ModuleRegistry() {}

    public static List<ModuleDefinition> all() {
        return MODULES;
    }
}
