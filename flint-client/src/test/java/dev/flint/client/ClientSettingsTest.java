package dev.flint.client;

import java.nio.file.Files;
import java.util.HashSet;

public final class ClientSettingsTest {
    public static void main(String[] args) throws Exception {
        var root = Files.createTempDirectory("flint-client-settings");
        var file = root.resolve("settings.json");
        ClientSettings defaults = ClientSettings.load(file);
        require(defaults.menuKey == 344, "Right Shift is the safe default");
        require(defaults.enabled("fps"), "FPS is enabled by default");
        require(!defaults.enabled("zoom"), "visual modules are opt-in");
        require(!defaults.enabled("auto_gg"), "chat automation is opt-in");
        var ids = new HashSet<String>();
        for (ModuleDefinition module : ModuleRegistry.all()) {
            require(ids.add(module.id()), "module IDs must be unique");
            require(defaults.modules.containsKey(module.id()), "every module has a persisted default");
        }
        defaults.modules.put("coordinates", true);
        defaults.hudPositions.put("fps", new ClientSettings.HudPosition(0.75f, 0.4f));
        ClientSettings.save(file, defaults);
        ClientSettings restored = ClientSettings.load(file);
        require(restored.enabled("coordinates"), "module state persists");
        require(Math.abs(restored.hudPosition("fps", 0).x - 0.75f) < 0.001f,
                "HUD position persists");
        require(FlintRuntime.isVictoryMessage("§6Victory!"), "formatted exact victory is recognized");
        require(!FlintRuntime.isVictoryMessage("Say victory! to claim a prize"),
                "embedded text cannot trigger Auto GG");
        Files.writeString(file, "not json");
        require(ClientSettings.load(file).enabled("fps"), "corrupt settings fail to defaults");
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }
}
