package dev.flint.client.mixin;

import dev.flint.client.AutoAuthClient;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientPlayNetworkHandler.class)
abstract class ClientPlayNetworkHandlerMixin {
    @Inject(method = "sendChatCommand", at = @At("HEAD"), cancellable = true)
    private void flint$interceptAutoAuth(String command, CallbackInfo callback) {
        if (AutoAuthClient.intercept(command, (ClientPlayNetworkHandler) (Object) this)) {
            callback.cancel();
        }
    }
}
