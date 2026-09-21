package dev.flint.client;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import net.fabricmc.loader.api.FabricLoader;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.lwjgl.glfw.GLFW;

import java.io.Reader;
import java.io.Writer;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.LinkedHashMap;
import java.util.Map;

public final class ClientSettings {
    private static final Logger LOGGER = LoggerFactory.getLogger("Flint Client");
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
    private static final int SCHEMA_VERSION = 1;
    private static final Path RELATIVE_PATH = Path.of("flint", "client-settings-v1.json");

    int schemaVersion = SCHEMA_VERSION;
    public int menuKey = GLFW.GLFW_KEY_RIGHT_SHIFT;
    public boolean reducedMotion;
    public Map<String, Boolean> modules = defaults();
    public Map<String, HudPosition> hudPositions = new LinkedHashMap<>();

    public boolean enabled(String id) {
        return Boolean.TRUE.equals(modules.get(id));
    }

    public void toggle(String id) {
        modules.put(id, !enabled(id));
        save();
    }

    public void setMenuKey(int key) {
        menuKey = key;
        save();
    }

    public HudPosition hudPosition(String id, int index) {
        return hudPositions.computeIfAbsent(id, ignored -> new HudPosition(0.02f, 0.03f + index * 0.055f));
    }

    public void setHudPosition(String id, float x, float y) {
        hudPositions.put(id, new HudPosition(clamp(x), clamp(y)));
        save();
    }

    public void save() {
        save(path(), this);
    }

    public static ClientSettings load() {
        return load(path());
    }

    static ClientSettings load(Path file) {
        if (!Files.isRegularFile(file)) {
            return new ClientSettings();
        }
        try (Reader reader = Files.newBufferedReader(file)) {
            ClientSettings loaded = GSON.fromJson(reader, ClientSettings.class);
            if (loaded == null || loaded.schemaVersion != SCHEMA_VERSION) {
                LOGGER.warn("Flint Client settings use an unsupported schema; safe defaults loaded");
                return new ClientSettings();
            }
            if (loaded.modules == null) loaded.modules = defaults();
            if (loaded.hudPositions == null) loaded.hudPositions = new LinkedHashMap<>();
            defaults().forEach(loaded.modules::putIfAbsent);
            if (loaded.menuKey < GLFW.GLFW_KEY_SPACE || loaded.menuKey > GLFW.GLFW_KEY_MENU) {
                loaded.menuKey = GLFW.GLFW_KEY_RIGHT_SHIFT;
            }
            return loaded;
        } catch (Exception error) {
            LOGGER.warn("Flint Client settings are unreadable; safe defaults loaded ({})",
                    error.getClass().getSimpleName());
            return new ClientSettings();
        }
    }

    static void save(Path file, ClientSettings settings) {
        try {
            Files.createDirectories(file.getParent());
            Path temporary = file.resolveSibling(file.getFileName() + ".tmp");
            try (Writer writer = Files.newBufferedWriter(temporary)) {
                GSON.toJson(settings, writer);
            }
            try {
                Files.move(temporary, file, StandardCopyOption.ATOMIC_MOVE,
                        StandardCopyOption.REPLACE_EXISTING);
            } catch (AtomicMoveNotSupportedException ignored) {
                Files.move(temporary, file, StandardCopyOption.REPLACE_EXISTING);
            }
        } catch (Exception error) {
            LOGGER.warn("Could not save Flint Client settings ({})", error.getClass().getSimpleName());
        }
    }

    private static Path path() {
        return FabricLoader.getInstance().getGameDir().toAbsolutePath().normalize().resolve(RELATIVE_PATH);
    }

    private static Map<String, Boolean> defaults() {
        Map<String, Boolean> values = new LinkedHashMap<>();
        values.put("fps", true);
        values.put("coordinates", false);
        values.put("ping", true);
        values.put("speed", false);
        values.put("memory", false);
        values.put("keystrokes", false);
        values.put("reach", false);
        values.put("armor", false);
        values.put("effects", false);
        values.put("cps", false);
        values.put("combo", false);
        values.put("fullbright", false);
        values.put("zoom", false);
        values.put("freelook", false);
        values.put("time_changer", false);
        values.put("weather_changer", false);
        values.put("custom_hand", false);
        values.put("projectile_trails", false);
        values.put("player_particles", false);
        values.put("audio_visualizer", false);
        values.put("toggle_sprint", false);
        values.put("auto_gg", false);
        return values;
    }

    private static float clamp(float value) {
        return Math.max(0.0f, Math.min(0.95f, value));
    }

    public static final class HudPosition {
        public float x;
        public float y;

        public HudPosition() {}

        HudPosition(float x, float y) {
            this.x = x;
            this.y = y;
        }
    }
}
