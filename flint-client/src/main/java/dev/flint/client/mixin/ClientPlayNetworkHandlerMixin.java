package dev.flint.client.mixin;

import dev.flint.client.AutoAuthClient;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.network.packet.s2c.play.GameJoinS2CPacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientPlayNetworkHandler.class)
abstract class ClientPlayNetworkHandlerMixin {
    @Inject(method = "onGameJoin", at = @At("HEAD"))
    private void flint$beginAutoAuthConnection(GameJoinS2CPacket packet, CallbackInfo callback) {
        AutoAuthClient.beginConnection();
    }

    @Inject(method = "sendChatCommand", at = @At("HEAD"), cancellable = true)
    private void flint$interceptAutoAuth(String command, CallbackInfo callback) {
        if (AutoAuthClient.intercept(command, (ClientPlayNetworkHandler) (Object) this)) {
            callback.cancel();
        }
    }
}
