package dev.flint.client;

import net.minecraft.client.sound.SoundInstance;

import java.lang.reflect.Proxy;
import java.util.concurrent.atomic.AtomicInteger;

public final class SoundVisualizerCaptureTest {
    public static void main(String[] args) {
        var volumesRead = new AtomicInteger();
        var recorded = new AtomicInteger();
        SoundInstance unresolved = (SoundInstance) Proxy.newProxyInstance(
                SoundInstance.class.getClassLoader(), new Class<?>[] {SoundInstance.class},
                (proxy, method, arguments) -> {
                    if (method.getName().equals("getSound")) return null;
                    if (method.getName().equals("getVolume")) {
                        volumesRead.incrementAndGet();
                        throw new AssertionError("unresolved sound volume must not be read");
                    }
                    throw new AssertionError("unexpected call: " + method.getName());
                });
        SoundVisualizerCapture.observe(false, unresolved, value -> recorded.incrementAndGet());
        SoundVisualizerCapture.observe(true, unresolved, value -> recorded.incrementAndGet());
        if (volumesRead.get() != 0 || recorded.get() != 0) {
            throw new AssertionError("disabled or unresolved sounds must not be sampled");
        }
    }
}
