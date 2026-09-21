package dev.flint.client.mixin;

import dev.flint.client.FlintRuntime;
import net.minecraft.client.sound.SoundInstance;
import net.minecraft.client.sound.SoundSystem;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(SoundSystem.class)
abstract class SoundSystemMixin {
    @Inject(method = "play(Lnet/minecraft/client/sound/SoundInstance;)Lnet/minecraft/client/sound/SoundSystem$PlayResult;",
            at = @At("HEAD"))
    private void flint$observeSound(SoundInstance sound,
                                    CallbackInfoReturnable<SoundSystem.PlayResult> callback) {
        FlintRuntime.recordSound(sound.getVolume());
    }
}
