package dev.flint.client;

import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

public final class HudEditorScreen extends Screen {
    private String dragging;
    private double dragOffsetX;
    private double dragOffsetY;

    public HudEditorScreen() {
        super(Text.literal("Edit HUD"));
    }

    @Override
    public boolean shouldPause() { return false; }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        context.fill(0, 0, width, 28, 0xE0101214);
        context.drawTextWithShadow(textRenderer, "EDIT HUD", 10, 10, FlintRuntime.AMBER);
        context.drawTextWithShadow(textRenderer, "Drag enabled elements  •  ESC to return", 72, 10, 0xFFAAB0A7);
        int index = 0;
        int visible = 0;
        for (ModuleDefinition module : FlintRuntime.hudModules()) {
            if (FlintRuntime.enabled(module.id())) {
                String value = FlintRuntime.hudValue(module.id(), client);
                if (value != null) {
                    FlintRuntime.renderHudItem(context, client, module.id(), value, index, true);
                    visible++;
                }
            }
            index++;
        }
        if (visible == 0) context.drawCenteredTextWithShadow(textRenderer,
                "Enable a HUD module first", width / 2, height / 2, 0xFFAAB0A7);
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        int index = 0;
        for (ModuleDefinition module : FlintRuntime.hudModules()) {
            if (FlintRuntime.enabled(module.id())) {
                String value = FlintRuntime.hudValue(module.id(), client);
                ClientSettings.HudPosition position = FlintRuntime.settings().hudPosition(module.id(), index);
                int x = Math.round(position.x * width);
                int y = Math.round(position.y * height);
                int itemWidth = textRenderer.getWidth(value) + 10;
                if (click.x() >= x && click.x() < x + itemWidth && click.y() >= y && click.y() < y + 16) {
                    dragging = module.id();
                    dragOffsetX = click.x() - x;
                    dragOffsetY = click.y() - y;
                    return true;
                }
            }
            index++;
        }
        return super.mouseClicked(click, doubled);
    }

    @Override
    public boolean mouseDragged(Click click, double deltaX, double deltaY) {
        if (dragging == null) return super.mouseDragged(click, deltaX, deltaY);
        ClientSettings.HudPosition position = FlintRuntime.settings().hudPositions.get(dragging);
        position.x = Math.max(0, Math.min(0.95f, (float) ((click.x() - dragOffsetX) / width)));
        position.y = Math.max(0.05f, Math.min(0.95f, (float) ((click.y() - dragOffsetY) / height)));
        return true;
    }

    @Override
    public boolean mouseReleased(Click click) {
        if (dragging != null) {
            dragging = null;
            FlintRuntime.settings().save();
            return true;
        }
        return super.mouseReleased(click);
    }

    @Override
    public void close() {
        client.setScreen(new FlintClientScreen());
    }
}
