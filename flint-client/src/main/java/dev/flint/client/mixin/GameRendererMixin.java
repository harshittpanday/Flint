package dev.flint.client.mixin;

import dev.flint.client.FlintRuntime;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.Camera;
import net.minecraft.client.render.GameRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(GameRenderer.class)
abstract class GameRendererMixin {
    @Inject(method = "getFov", at = @At("RETURN"), cancellable = true)
    private void flint$applyZoom(Camera camera, float tickProgress, boolean changingFov,
                                 CallbackInfoReturnable<Float> callback) {
        if (FlintRuntime.zoomActive(MinecraftClient.getInstance())) {
            callback.setReturnValue(Math.min(callback.getReturnValue(), 30.0f));
        }
    }
}
