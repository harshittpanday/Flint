package dev.flint.client.mixin;

import dev.flint.client.FlintClient;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.entity.player.SkinTextures;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(AbstractClientPlayerEntity.class)
abstract class AbstractClientPlayerEntityMixin {
    @Inject(method = "getSkin", at = @At("RETURN"), cancellable = true)
    private void flint$overrideLocalSkin(CallbackInfoReturnable<SkinTextures> callback) {
        SkinTextures original = callback.getReturnValue();
        if (original != null) {
            callback.setReturnValue(FlintClient.overrideLocalSkin(
                    (AbstractClientPlayerEntity) (Object) this,
                    original
            ));
        }
    }
}
