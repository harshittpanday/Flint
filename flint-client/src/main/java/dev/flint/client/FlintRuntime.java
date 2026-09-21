package dev.flint.client;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.item.ItemStack;
import net.minecraft.entity.EquipmentSlot;

import java.util.Locale;

public final class FlintRuntime {
    private static ClientSettings settings = new ClientSettings();
    private static double previousX;
    private static double previousZ;
    private static double speed;
    private static Double originalGamma;

    private FlintRuntime() {}

    public static void initialize() {
        settings = ClientSettings.load();
    }

    public static ClientSettings settings() {
        return settings;
    }

    public static void tick(MinecraftClient client) {
        if (client.player == null) return;
        double dx = client.player.getX() - previousX;
        double dz = client.player.getZ() - previousZ;
        speed = Math.sqrt(dx * dx + dz * dz) * 20.0;
        previousX = client.player.getX();
        previousZ = client.player.getZ();
        boolean fullbright = settings.enabled("fullbright");
        if (fullbright && originalGamma == null) {
            originalGamma = client.options.getGamma().getValue();
            client.options.getGamma().setValue(16.0);
        } else if (!fullbright && originalGamma != null) {
            client.options.getGamma().setValue(originalGamma);
            originalGamma = null;
        }
    }

    public static void toggleMenu(MinecraftClient client) {
        if (client.currentScreen instanceof FlintClientScreen) {
            client.setScreen(null);
        } else if (client.currentScreen == null) {
            client.setScreen(new FlintClientScreen());
        }
    }

    public static void renderHud(DrawContext context, MinecraftClient client) {
        if (client.player == null || client.options.hudHidden || client.currentScreen instanceof FlintClientScreen) return;
        int x = 8;
        int y = 8;
        for (ModuleDefinition module : ModuleRegistry.all()) {
            if (!settings.enabled(module.id())) continue;
            String value = value(module.id(), client);
            if (value == null) continue;
            int width = client.textRenderer.getWidth(value) + 10;
            context.fill(x, y, x + width, y + 16, 0xB015181A);
            context.fill(x, y, x + 2, y + 16, 0xFF9BEA3A);
            context.drawTextWithShadow(client.textRenderer, value, x + 6, y + 4, 0xFFF3F6EF);
            y += 20;
        }
    }

    private static String value(String id, MinecraftClient client) {
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
            default -> null;
        };
    }

    private static String key(boolean pressed, String label) {
        return pressed ? "[" + label + "]" : label;
    }

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
}
