package dev.flint.client.mixin;

import dev.flint.client.FlintRuntime;
import net.minecraft.client.Keyboard;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.input.KeyInput;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Keyboard.class)
abstract class KeyboardMixin {
    @Inject(method = "onKey", at = @At("HEAD"), cancellable = true)
    private void flint$handleMenuKey(long window, int action, KeyInput input, CallbackInfo callback) {
        if (action == 1 && input.key() == FlintRuntime.settings().menuKey) {
            FlintRuntime.toggleMenu(MinecraftClient.getInstance());
            callback.cancel();
        }
    }
}
