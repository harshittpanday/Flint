package dev.flint.client.mixin;

import dev.flint.client.AutoAuthClient;
import dev.flint.client.FlintRuntime;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.network.packet.s2c.play.GameJoinS2CPacket;
import net.minecraft.network.packet.s2c.play.GameMessageS2CPacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientPlayNetworkHandler.class)
abstract class ClientPlayNetworkHandlerMixin {
    @Inject(method = "onGameJoin", at = @At("HEAD"))
    private void flint$beginAutoAuthConnection(GameJoinS2CPacket packet, CallbackInfo callback) {
        AutoAuthClient.beginConnection();
        FlintRuntime.beginConnection();
    }

    @Inject(method = "onGameMessage", at = @At("TAIL"))
    private void flint$observeVictoryMessage(GameMessageS2CPacket packet, CallbackInfo callback) {
        FlintRuntime.onGameMessage(packet.content(), packet.overlay());
    }
}
