package dev.flint.client;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.client.util.InputUtil;
import net.minecraft.entity.Entity;
import net.minecraft.entity.EquipmentSlot;
import net.minecraft.entity.effect.StatusEffectInstance;
import net.minecraft.entity.projectile.ProjectileEntity;
import net.minecraft.item.ItemStack;
import net.minecraft.particle.ParticleTypes;
import net.minecraft.text.Text;
import net.minecraft.util.hit.EntityHitResult;

import java.util.ArrayDeque;
import java.util.Deque;
import java.util.List;
import java.util.Locale;

public final class FlintRuntime {
    public static final int AMBER = 0xFFFF9F1C;
    private static ClientSettings settings = new ClientSettings();
    private static final Deque<Long> LEFT_CLICKS = new ArrayDeque<>();
    private static final Deque<Long> RIGHT_CLICKS = new ArrayDeque<>();
    private static double previousX;
    private static double previousZ;
    private static double speed;
    private static Double originalGamma;
    private static Long originalTime;
    private static long timeOverrideStarted;
    private static Float originalRain;
    private static Float originalThunder;
    private static boolean sprintKeyDown;
    private static boolean sprintLatched;
    private static boolean freelookActive;
    private static float freelookYaw;
    private static float freelookPitch;
    private static int runtimeTicks;
    private static int combo;
    private static long lastComboAt;
    private static int autoGgDelay = -1;
    private static boolean autoGgSent;
    private static long lastAutoGgAt;

    private FlintRuntime() {}

    public static void initialize() {
        settings = ClientSettings.load();
    }

    public static ClientSettings settings() {
        return settings;
    }

    public static boolean enabled(String id) {
        return settings.enabled(id);
    }

    public static void beginConnection() {
        autoGgDelay = -1;
        autoGgSent = false;
        combo = 0;
        sprintLatched = false;
        originalTime = null;
        originalRain = null;
        originalThunder = null;
    }

    public static void tick(MinecraftClient client) {
        runtimeTicks++;
        if (client.player == null || client.world == null) return;
        double dx = client.player.getX() - previousX;
        double dz = client.player.getZ() - previousZ;
        speed = Math.sqrt(dx * dx + dz * dz) * 20.0;
        if (speed > 50.0) speed = 0.0;
        previousX = client.player.getX();
        previousZ = client.player.getZ();
        updateFullbright(client);
        updateFreelook(client);
        updateTimeAndWeather(client);
        updateToggleSprint(client);
        updateParticles(client, dx, dz);
        updateAutoGg(client);
        pruneClicks(System.currentTimeMillis());
        if (client.player.hurtTime > 0 || System.currentTimeMillis() - lastComboAt > 3000) combo = 0;
    }

    private static void updateFullbright(MinecraftClient client) {
        if (enabled("fullbright") && originalGamma == null) {
            originalGamma = client.options.getGamma().getValue();
            client.options.getGamma().setValue(16.0);
        } else if (!enabled("fullbright") && originalGamma != null) {
            client.options.getGamma().setValue(originalGamma);
            originalGamma = null;
        }
    }

    private static void updateFreelook(MinecraftClient client) {
        boolean held = enabled("freelook") && InputUtil.isKeyPressed(client.getWindow(), 342);
        if (held && !freelookActive) {
            freelookYaw = client.player.getYaw();
            freelookPitch = client.player.getPitch();
        }
        freelookActive = held;
    }

    private static void updateTimeAndWeather(MinecraftClient client) {
        if (enabled("time_changer")) {
            if (originalTime == null) {
                originalTime = client.world.getTimeOfDay();
                timeOverrideStarted = runtimeTicks;
            }
            client.world.setTime(client.world.getTime(), 6000L, false);
        } else if (originalTime != null) {
            client.world.setTime(client.world.getTime(), originalTime + runtimeTicks - timeOverrideStarted, false);
            originalTime = null;
        }
        if (enabled("weather_changer")) {
            if (originalRain == null) {
                originalRain = client.world.getRainGradient(1.0f);
                originalThunder = client.world.getThunderGradient(1.0f);
            }
            client.world.setRainGradient(0.0f);
            client.world.setThunderGradient(0.0f);
        } else if (originalRain != null) {
            client.world.setRainGradient(originalRain);
            client.world.setThunderGradient(originalThunder == null ? 0.0f : originalThunder);
            originalRain = null;
            originalThunder = null;
        }
    }

    private static void updateToggleSprint(MinecraftClient client) {
        boolean pressed = client.options.sprintKey.isPressed();
        if (enabled("toggle_sprint") && pressed && !sprintKeyDown) sprintLatched = !sprintLatched;
        sprintKeyDown = pressed;
        if (!enabled("toggle_sprint")) {
            if (sprintLatched) client.player.setSprinting(false);
            sprintLatched = false;
            return;
        }
        if (sprintLatched) {
            client.player.setSprinting(client.options.forwardKey.isPressed() && !client.player.isSneaking());
        }
    }

    private static void updateParticles(MinecraftClient client, double dx, double dz) {
        if (enabled("player_particles") && runtimeTicks % 4 == 0 && dx * dx + dz * dz > 0.0025) {
            client.world.addParticleClient(ParticleTypes.FLAME,
                    client.player.getX(), client.player.getY() + 0.1, client.player.getZ(), 0, 0.01, 0);
        }
        if (!enabled("projectile_trails") || runtimeTicks % 2 != 0) return;
        int rendered = 0;
        for (Entity entity : client.world.getEntities()) {
            if (entity instanceof ProjectileEntity
                    && entity.squaredDistanceTo(client.player) < 4096
                    && rendered++ < 32) {
                client.world.addParticleClient(ParticleTypes.FLAME,
                        entity.getX(), entity.getY(), entity.getZ(), 0, 0, 0);
            }
        }
    }

    private static void updateAutoGg(MinecraftClient client) {
        if (!enabled("auto_gg")) {
            autoGgDelay = -1;
            return;
        }
        if (autoGgDelay > 0) autoGgDelay--;
        if (autoGgDelay == 0 && !autoGgSent && client.getNetworkHandler() != null) {
            client.getNetworkHandler().sendChatMessage("gg");
            autoGgSent = true;
            autoGgDelay = -1;
            lastAutoGgAt = System.currentTimeMillis();
        }
    }

    public static void onGameMessage(Text message, boolean overlay) {
        if (overlay || !enabled("auto_gg") || autoGgSent || autoGgDelay >= 0) return;
        if (isVictoryMessage(message.getString())
                && System.currentTimeMillis() - lastAutoGgAt >= 10_000) {
            autoGgDelay = 20;
        }
    }

    static boolean isVictoryMessage(String message) {
        String normalized = message.replaceAll("§.", "").trim().toLowerCase(Locale.ROOT);
        return List.of("victory!", "you won!", "winner!").contains(normalized);
    }

    public static void recordMouseClick(int button) {
        long now = System.currentTimeMillis();
        (button == 0 ? LEFT_CLICKS : RIGHT_CLICKS).addLast(now);
        pruneClicks(now);
    }

    public static void recordAttack(MinecraftClient client, boolean successful) {
        if (successful && client.crosshairTarget instanceof EntityHitResult) {
            combo++;
            lastComboAt = System.currentTimeMillis();
        }
    }

    public static boolean zoomActive(MinecraftClient client) {
        return enabled("zoom") && InputUtil.isKeyPressed(client.getWindow(), 67);
    }

    public static boolean freelookActive() { return freelookActive; }
    public static float freelookYaw() { return freelookYaw; }
    public static float freelookPitch() { return freelookPitch; }

    public static void applyFreelookDelta(double x, double y) {
        freelookYaw += (float) (x * 0.15);
        freelookPitch = Math.max(-90.0f, Math.min(90.0f, freelookPitch + (float) (y * 0.15)));
    }

    public static void toggleMenu(MinecraftClient client) {
        if (client.currentScreen instanceof FlintClientScreen || client.currentScreen instanceof HudEditorScreen) {
            client.setScreen(null);
        }
        else if (client.currentScreen == null) client.setScreen(new FlintClientScreen());
    }

    public static List<ModuleDefinition> hudModules() {
        return ModuleRegistry.all().stream()
                .filter(module -> module.category() == ModuleDefinition.Category.HUD).toList();
    }

    public static void renderHud(DrawContext context, MinecraftClient client) {
        if (client.player == null || client.options.hudHidden
                || client.currentScreen instanceof FlintClientScreen
                || client.currentScreen instanceof HudEditorScreen) return;
        int index = 0;
        for (ModuleDefinition module : hudModules()) {
            if (enabled(module.id())) {
                String value = hudValue(module.id(), client);
                if (value != null) renderHudItem(context, client, module.id(), value, index, false);
            }
            index++;
        }
    }

    public static void renderHudItem(DrawContext context, MinecraftClient client, String id,
                                     String value, int index, boolean editing) {
        ClientSettings.HudPosition position = settings.hudPosition(id, index);
        int x = Math.round(position.x * context.getScaledWindowWidth());
        int y = Math.round(position.y * context.getScaledWindowHeight());
        int width = client.textRenderer.getWidth(value) + 10;
        context.fill(x, y, x + width, y + 16, editing ? 0xE02A221A : 0xB015181A);
        context.fill(x, y, x + 2, y + 16, AMBER);
        context.drawTextWithShadow(client.textRenderer, value, x + 6, y + 4, 0xFFF3F6EF);
    }

    public static String hudValue(String id, MinecraftClient client) {
        return switch (id) {
            case "fps" -> "FPS  " + client.getCurrentFps();
            case "coordinates" -> String.format(Locale.ROOT, "XYZ  %.1f  %.1f  %.1f",
                    client.player.getX(), client.player.getY(), client.player.getZ());
            case "ping" -> {
                PlayerListEntry entry = client.getNetworkHandler() == null ? null
                        : client.getNetworkHandler().getPlayerListEntry(client.player.getUuid());
                yield entry == null ? "PING  --" : "PING  " + entry.getLatency() + " ms";
            }
            case "speed" -> String.format(Locale.ROOT, "SPEED  %.2f m/s", speed);
            case "memory" -> {
                Runtime runtime = Runtime.getRuntime();
                long used = (runtime.totalMemory() - runtime.freeMemory()) / 1024 / 1024;
                long maximum = runtime.maxMemory() / 1024 / 1024;
                yield "MEM  " + used + " / " + maximum + " MB";
            }
            case "keystrokes" -> "KEYS  "
                    + key(client.options.forwardKey.isPressed(), "W") + " "
                    + key(client.options.leftKey.isPressed(), "A") + " "
                    + key(client.options.backKey.isPressed(), "S") + " "
                    + key(client.options.rightKey.isPressed(), "D") + "  "
                    + key(client.options.jumpKey.isPressed(), "SPACE");
            case "reach" -> client.crosshairTarget == null ? "REACH  --" : String.format(
                    Locale.ROOT, "REACH  %.2f m",
                    Math.sqrt(client.player.getEyePos().squaredDistanceTo(client.crosshairTarget.getPos())));
            case "armor" -> armor(client);
            case "effects" -> effects(client);
            case "cps" -> "CPS  " + LEFT_CLICKS.size() + " | " + RIGHT_CLICKS.size();
            case "combo" -> "COMBO  " + combo;
            default -> null;
        };
    }

    private static void pruneClicks(long now) {
        while (!LEFT_CLICKS.isEmpty() && now - LEFT_CLICKS.peekFirst() > 1000) LEFT_CLICKS.removeFirst();
        while (!RIGHT_CLICKS.isEmpty() && now - RIGHT_CLICKS.peekFirst() > 1000) RIGHT_CLICKS.removeFirst();
    }

    private static String key(boolean pressed, String label) { return pressed ? "[" + label + "]" : label; }

    private static String armor(MinecraftClient client) {
        int remaining = 0;
        int maximum = 0;
        for (EquipmentSlot slot : new EquipmentSlot[]{
                EquipmentSlot.FEET, EquipmentSlot.LEGS, EquipmentSlot.CHEST, EquipmentSlot.HEAD}) {
            ItemStack stack = client.player.getEquippedStack(slot);
            if (!stack.isEmpty() && stack.isDamageable()) {
                remaining += stack.getMaxDamage() - stack.getDamage();
                maximum += stack.getMaxDamage();
            }
        }
        return maximum == 0 ? "ARMOR  --" : "ARMOR  " + (remaining * 100 / maximum) + "%";
    }

    private static String effects(MinecraftClient client) {
        if (client.player.getStatusEffects().isEmpty()) return "EFFECTS  none";
        return "EFFECTS  " + client.player.getStatusEffects().stream().limit(3)
                .map(FlintRuntime::effectLabel).reduce((a, b) -> a + " · " + b).orElse("none");
    }

    private static String effectLabel(StatusEffectInstance effect) {
        int seconds = Math.max(0, effect.getDuration() / 20);
        return effect.getEffectType().value().getName().getString() + " "
                + (effect.getAmplifier() + 1) + " " + seconds + "s";
    }
}
