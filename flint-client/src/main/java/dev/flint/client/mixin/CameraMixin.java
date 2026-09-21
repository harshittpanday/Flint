package dev.flint.client.mixin;

import dev.flint.client.FlintRuntime;
import net.minecraft.client.render.Camera;
import net.minecraft.entity.Entity;
import net.minecraft.world.World;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Camera.class)
abstract class CameraMixin {
    @Shadow protected abstract void setRotation(float yaw, float pitch);

    @Inject(method = "update", at = @At("TAIL"))
    private void flint$applyFreelookCamera(World world, Entity focusedEntity, boolean thirdPerson,
                                           boolean inverseView, float tickProgress, CallbackInfo callback) {
        if (FlintRuntime.freelookActive()) {
            setRotation(FlintRuntime.freelookYaw(), FlintRuntime.freelookPitch());
        }
    }
}
