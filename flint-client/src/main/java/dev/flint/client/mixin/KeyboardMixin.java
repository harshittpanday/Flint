package dev.flint.client.mixin;

import dev.flint.client.FlintRuntime;
import net.minecraft.client.Keyboard;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.input.KeyInput;
import org.lwjgl.glfw.GLFW;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Keyboard.class)
abstract class KeyboardMixin {
    @Unique private static final Logger FLINT_LOGGER = LoggerFactory.getLogger("Flint Client");

    static {
        FLINT_LOGGER.info("Flint Client keyboard hook loaded");
    }

    @Inject(method = "onKey", at = @At("HEAD"), cancellable = true)
    private void flint$handleMenuKey(long window, int action, KeyInput input, CallbackInfo callback) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (window == client.getWindow().getHandle()
                && action == GLFW.GLFW_PRESS
                && input.key() == FlintRuntime.settings().menuKey) {
            FLINT_LOGGER.info("Flint Client menu key pressed; requesting screen");
            FlintRuntime.requestMenuToggle();
            callback.cancel();
        }
    }
}
