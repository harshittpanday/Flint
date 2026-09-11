package dev.flint.client;

final class ClientConfig {
    int protocolVersion;
    boolean enabled;
    Cosmetics cosmetics = new Cosmetics();

    static final class Cosmetics {
        boolean skinEnabled;
        String skinPath;
        String skinModel = "classic";
        boolean capeEnabled;
        String capePath;
    }
}
