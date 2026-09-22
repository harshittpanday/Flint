package dev.flint.client;

import net.minecraft.client.sound.SoundInstance;

import java.util.function.Consumer;

public final class SoundVisualizerCapture {
    private SoundVisualizerCapture() {}

    public static void observe(boolean enabled, SoundInstance sound, Consumer<Float> record) {
        // Minecraft resolves AbstractSoundInstance.sound inside SoundSystem.play, not before it.
        if (enabled && sound != null && sound.getSound() != null) {
            record.accept(sound.getVolume());
        }
    }
}
