package dev.flint.client;

import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.input.KeyInput;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.text.Text;

import java.util.List;

public final class FlintClientScreen extends Screen {
    private static final int AMBER = FlintRuntime.AMBER;
    private static final int TEXT = 0xFFF3F6EF;
    private ModuleDefinition.Category category = ModuleDefinition.Category.HUD;
    private boolean capturingKey;
    private String keyWarning;
    private int scrollOffset;

    public FlintClientScreen() {
        super(Text.literal("Flint Client"));
    }

    @Override
    public boolean shouldPause() {
        return false;
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        int panelWidth = Math.min(760, width - 32);
        int panelHeight = Math.min(480, height - 32);
        int left = (width - panelWidth) / 2;
        int top = (height - panelHeight) / 2;
        int contentLeft = left + 194;
        context.fill(0, 0, width, height, 0xA0000000);
        context.fill(left, top, left + panelWidth, top + panelHeight, 0xFA101214);
        context.fill(left, top, left + 168, top + panelHeight, 0xFF171A1D);
        context.fill(left, top, left + panelWidth, top + 2, AMBER);
        context.drawTextWithShadow(textRenderer, "FLINT", left + 22, top + 22, AMBER);
        context.drawTextWithShadow(textRenderer, "CLIENT", left + 58, top + 22, 0xFFAAB0A7);
        context.drawTextWithShadow(textRenderer, "Right Shift  •  ESC to close", left + 22,
                top + panelHeight - 24, 0xFF70766E);

        int categoryY = top + 62;
        for (ModuleDefinition.Category item : ModuleDefinition.Category.values()) {
            boolean selected = item == category;
            if (selected) context.fill(left + 10, categoryY - 7, left + 158, categoryY + 17, 0xFF252B25);
            if (selected) context.fill(left + 10, categoryY - 7, left + 13, categoryY + 17, AMBER);
            context.drawTextWithShadow(textRenderer, label(item), left + 22, categoryY,
                    selected ? TEXT : 0xFF8F958C);
            categoryY += 34;
        }

        context.drawTextWithShadow(textRenderer, label(category), contentLeft, top + 24, TEXT);
        context.drawTextWithShadow(textRenderer, subtitle(category), contentLeft, top + 42, 0xFF858B82);
        List<ModuleDefinition> modules = ModuleRegistry.all().stream()
                .filter(module -> module.category() == category).toList();
        if (category == ModuleDefinition.Category.HUD) {
            context.fill(contentLeft, top + 70, contentLeft + 116, top + 94,
                    hovered(mouseX, mouseY, contentLeft, top + 70, contentLeft + 116, top + 94)
                            ? 0xFF362A1D : 0xFF28221C);
            context.drawTextWithShadow(textRenderer, "Edit HUD", contentLeft + 34, top + 78, AMBER);
        }
        int listTop = top + (category == ModuleDefinition.Category.HUD ? 108 : 76);
        int cardY = listTop - scrollOffset;
        context.enableScissor(contentLeft, listTop, left + panelWidth - 20, top + panelHeight - 16);
        if (modules.isEmpty()) {
            if (category == ModuleDefinition.Category.SETTINGS) {
                String key = capturingKey ? "Press a key…" : InputUtil.fromKeyCode(
                        new KeyInput(FlintRuntime.settings().menuKey, 0, 0)).getLocalizedText().getString();
                int right = left + panelWidth - 24;
                context.fill(contentLeft, cardY, right, cardY + 54, 0xFF1C2022);
                context.drawTextWithShadow(textRenderer, "Menu key", contentLeft + 14, cardY + 11, TEXT);
                context.drawTextWithShadow(textRenderer, key, right - textRenderer.getWidth(key) - 14,
                        cardY + 22, capturingKey ? AMBER : 0xFFAAB0A7);
                if (keyWarning != null) context.drawTextWithShadow(textRenderer, keyWarning,
                        contentLeft, cardY + 66, 0xFFFFA45B);
            } else {
                context.drawTextWithShadow(textRenderer, "No configurable modules in this category yet.",
                        contentLeft, cardY, 0xFF858B82);
            }
        }
        for (ModuleDefinition module : modules) {
            boolean enabled = FlintRuntime.settings().enabled(module.id());
            int right = left + panelWidth - 24;
            context.fill(contentLeft, cardY, right, cardY + 54,
                    hovered(mouseX, mouseY, contentLeft, cardY, right, cardY + 54)
                            ? 0xFF24282B : 0xFF1C2022);
            context.drawTextWithShadow(textRenderer, module.name(), contentLeft + 14, cardY + 11, TEXT);
            context.drawTextWithShadow(textRenderer, module.description(), contentLeft + 14, cardY + 30,
                    0xFF858B82);
            int toggleLeft = right - 44;
            context.fill(toggleLeft, cardY + 17, right - 12, cardY + 35,
                    enabled ? 0xFFD97706 : 0xFF353A3D);
            context.fill(enabled ? right - 27 : toggleLeft + 3, cardY + 20,
                    enabled ? right - 15 : toggleLeft + 15, cardY + 32, 0xFFF3F6EF);
            cardY += 66;
        }
        context.disableScissor();
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        int panelWidth = Math.min(760, width - 32);
        int panelHeight = Math.min(480, height - 32);
        int left = (width - panelWidth) / 2;
        int top = (height - panelHeight) / 2;
        int contentLeft = left + 194;
        int categoryY = top + 55;
        for (ModuleDefinition.Category item : ModuleDefinition.Category.values()) {
            if (hovered(click.x(), click.y(), left + 10, categoryY, left + 158, categoryY + 24)) {
                category = item;
                scrollOffset = 0;
                return true;
            }
            categoryY += 34;
        }
        if (category == ModuleDefinition.Category.HUD
                && hovered(click.x(), click.y(), contentLeft, top + 70, contentLeft + 116, top + 94)) {
            client.setScreen(new HudEditorScreen());
            return true;
        }
        int cardY = top + (category == ModuleDefinition.Category.HUD ? 108 : 76) - scrollOffset;
        if (category == ModuleDefinition.Category.SETTINGS
                && hovered(click.x(), click.y(), contentLeft, cardY, left + panelWidth - 24, cardY + 54)) {
            capturingKey = true;
            keyWarning = null;
            return true;
        }
        for (ModuleDefinition module : ModuleRegistry.all().stream()
                .filter(value -> value.category() == category).toList()) {
            if (hovered(click.x(), click.y(), contentLeft, cardY, left + panelWidth - 24, cardY + 54)) {
                FlintRuntime.settings().toggle(module.id());
                return true;
            }
            cardY += 66;
        }
        return super.mouseClicked(click, doubled);
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double horizontal, double vertical) {
        int count = (int) ModuleRegistry.all().stream().filter(value -> value.category() == category).count();
        int maximum = Math.max(0, count * 66 - 340);
        scrollOffset = Math.max(0, Math.min(maximum, scrollOffset - (int) (vertical * 28)));
        return maximum > 0 || super.mouseScrolled(mouseX, mouseY, horizontal, vertical);
    }

    @Override
    public boolean keyPressed(KeyInput input) {
        if (capturingKey) {
            if (input.key() == 256) {
                capturingKey = false;
                return true;
            }
            for (KeyBinding binding : client.options.allKeys) {
                if (binding.matchesKey(input)) {
                    keyWarning = "Already used by Minecraft: " + binding.getBoundKeyLocalizedText().getString();
                    return true;
                }
            }
            FlintRuntime.settings().setMenuKey(input.key());
            capturingKey = false;
            keyWarning = null;
            return true;
        }
        if (input.key() == FlintRuntime.settings().menuKey) {
            close();
            return true;
        }
        return super.keyPressed(input);
    }

    private static boolean hovered(double x, double y, int left, int top, int right, int bottom) {
        return x >= left && x < right && y >= top && y < bottom;
    }

    private static String label(ModuleDefinition.Category value) {
        String name = value.name().toLowerCase();
        return Character.toUpperCase(name.charAt(0)) + name.substring(1);
    }

    private static String subtitle(ModuleDefinition.Category value) {
        return value == ModuleDefinition.Category.HUD
                ? "Choose the information shown while you play."
                : "Additional focused modules will appear here when available.";
    }
}
