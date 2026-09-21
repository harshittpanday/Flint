package dev.flint.client.mixin;

import dev.flint.client.FlintRuntime;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.render.command.OrderedRenderCommandQueue;
import net.minecraft.client.render.item.HeldItemRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.item.ItemStack;
import net.minecraft.util.Hand;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(HeldItemRenderer.class)
abstract class HeldItemRendererMixin {
    @Inject(method = "renderFirstPersonItem", at = @At("HEAD"))
    private void flint$pushCustomHand(AbstractClientPlayerEntity player, float tickProgress, float pitch,
                                      Hand hand, float swingProgress, ItemStack item, float equipProgress,
                                      MatrixStack matrices, OrderedRenderCommandQueue queue, int light,
                                      CallbackInfo callback) {
        if (FlintRuntime.enabled("custom_hand")) {
            matrices.push();
            matrices.translate(0.08f, -0.08f, -0.12f);
            matrices.scale(0.88f, 0.88f, 0.88f);
        }
    }

    @Inject(method = "renderFirstPersonItem", at = @At("RETURN"))
    private void flint$popCustomHand(AbstractClientPlayerEntity player, float tickProgress, float pitch,
                                     Hand hand, float swingProgress, ItemStack item, float equipProgress,
                                     MatrixStack matrices, OrderedRenderCommandQueue queue, int light,
                                     CallbackInfo callback) {
        if (FlintRuntime.enabled("custom_hand")) matrices.pop();
    }
}
