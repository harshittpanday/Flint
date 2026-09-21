package dev.flint.client.mixin;

import dev.flint.client.AutoAuthClient;
import dev.flint.client.FlintRuntime;
import net.minecraft.client.MinecraftClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(MinecraftClient.class)
abstract class MinecraftClientMixin {
    @Inject(method = "tick", at = @At("TAIL"))
    private void flint$runAutoAuthWhenReady(CallbackInfo callback) {
        AutoAuthClient.tick((MinecraftClient) (Object) this);
        FlintRuntime.tick((MinecraftClient) (Object) this);
    }

    @Inject(method = "doAttack", at = @At("RETURN"))
    private void flint$recordCombo(CallbackInfoReturnable<Boolean> callback) {
        FlintRuntime.recordAttack((MinecraftClient) (Object) this, callback.getReturnValue());
    }
}
