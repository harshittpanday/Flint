package dev.flint.client.mixin;

import dev.flint.client.FlintRuntime;
import net.minecraft.client.Mouse;
import net.minecraft.client.input.MouseInput;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Mouse.class)
abstract class MouseMixin {
    @Shadow private double cursorDeltaX;
    @Shadow private double cursorDeltaY;

    @Inject(method = "updateMouse", at = @At("HEAD"), cancellable = true)
    private void flint$applyFreelook(double timeDelta, CallbackInfo callback) {
        if (!FlintRuntime.freelookActive()) return;
        FlintRuntime.applyFreelookDelta(cursorDeltaX, cursorDeltaY);
        cursorDeltaX = 0;
        cursorDeltaY = 0;
        callback.cancel();
    }

    @Inject(method = "onMouseButton", at = @At("HEAD"))
    private void flint$recordClick(long window, MouseInput input, int action, CallbackInfo callback) {
        if (action == 1 && (input.button() == 0 || input.button() == 1)) {
            FlintRuntime.recordMouseClick(input.button());
        }
    }
}
