package dev.flint.client;

import java.nio.file.Files;

public final class ClientSettingsTest {
    public static void main(String[] args) throws Exception {
        var root = Files.createTempDirectory("flint-client-settings");
        var file = root.resolve("settings.json");
        ClientSettings defaults = ClientSettings.load(file);
        require(defaults.menuKey == 344, "Right Shift is the safe default");
        require(defaults.enabled("fps"), "FPS is enabled by default");
        defaults.modules.put("coordinates", true);
        ClientSettings.save(file, defaults);
        ClientSettings restored = ClientSettings.load(file);
        require(restored.enabled("coordinates"), "module state persists");
        Files.writeString(file, "not json");
        require(ClientSettings.load(file).enabled("fps"), "corrupt settings fail to defaults");
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }
}
