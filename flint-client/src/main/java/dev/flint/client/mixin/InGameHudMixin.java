package dev.flint.client.mixin;

import dev.flint.client.FlintRuntime;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.client.render.RenderTickCounter;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
abstract class InGameHudMixin {
    @Inject(method = "render", at = @At("TAIL"))
    private void flint$renderHud(DrawContext context, RenderTickCounter tickCounter, CallbackInfo callback) {
        FlintRuntime.renderHud(context, MinecraftClient.getInstance());
    }
}
