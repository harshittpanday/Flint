package dev.flint.client;

import com.google.gson.Gson;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.entity.player.PlayerSkinType;
import net.minecraft.entity.player.SkinTextures;
import net.minecraft.util.AssetInfo;
import net.minecraft.util.Identifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.io.Reader;
import java.nio.file.Files;
import java.nio.file.Path;

public final class FlintClient implements ClientModInitializer {
    private static final Logger LOGGER = LoggerFactory.getLogger("Flint Client");
    private static final int PROTOCOL_VERSION = 1;
    private static final Path CONFIG = Path.of("flint", "client-v1.json");
    private static final Identifier SKIN_ID = Identifier.of("flint", "local_skin");
    private static final Identifier CAPE_ID = Identifier.of("flint", "local_cape");
    private static ClientConfig config;
    private static AssetInfo.TextureAsset skin;
    private static AssetInfo.TextureAsset cape;
    private static boolean texturesLoaded;

    @Override
    public void onInitializeClient() {
        config = loadConfig();
        if (config == null || !config.enabled) {
            LOGGER.info("Flint Client is disabled for this profile");
            return;
        }
        LOGGER.info("Flint Client protocol {} initialized", PROTOCOL_VERSION);
    }

    public static SkinTextures overrideLocalSkin(AbstractClientPlayerEntity player, SkinTextures original) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (config == null || !config.enabled || client.player != player) {
            return original;
        }
        loadTextures(client);
        AssetInfo.TextureAsset body = skin != null ? skin : original.body();
        AssetInfo.TextureAsset localCape = cape != null ? cape : original.cape();
        PlayerSkinType model = skin == null
                ? original.model()
                : "slim".equals(config.cosmetics.skinModel) ? PlayerSkinType.SLIM : PlayerSkinType.WIDE;
        return new SkinTextures(body, localCape, original.elytra(), model, false);
    }

    private static ClientConfig loadConfig() {
        Path gameDir = FabricLoader.getInstance().getGameDir().toAbsolutePath().normalize();
        Path configPath = gameDir.resolve(CONFIG).normalize();
        if (!configPath.startsWith(gameDir) || !Files.isRegularFile(configPath)) {
            return null;
        }
        try (Reader reader = Files.newBufferedReader(configPath)) {
            ClientConfig loaded = new Gson().fromJson(reader, ClientConfig.class);
            if (loaded == null || loaded.protocolVersion != PROTOCOL_VERSION) {
                LOGGER.warn("Unsupported or missing Flint Client protocol configuration");
                return null;
            }
            return loaded;
        } catch (IOException | RuntimeException error) {
            LOGGER.warn("Could not read Flint Client configuration: {}", error.getClass().getSimpleName());
            return null;
        }
    }

    private static synchronized void loadTextures(MinecraftClient client) {
        if (texturesLoaded) {
            return;
        }
        texturesLoaded = true;
        if (config.cosmetics == null) {
            return;
        }
        if (config.cosmetics.skinEnabled) {
            skin = loadTexture(client, config.cosmetics.skinPath, SKIN_ID, "skin");
        }
        if (config.cosmetics.capeEnabled) {
            cape = loadTexture(client, config.cosmetics.capePath, CAPE_ID, "cape");
        }
    }

    private static AssetInfo.TextureAsset loadTexture(
            MinecraftClient client,
            String relativeValue,
            Identifier id,
            String kind
    ) {
        if (relativeValue == null) {
            return null;
        }
        Path gameDir = FabricLoader.getInstance().getGameDir().toAbsolutePath().normalize();
        Path relative = Path.of(relativeValue);
        if (relative.isAbsolute() || relative.normalize().startsWith("..")) {
            LOGGER.warn("Rejected unsafe Flint local {} path", kind);
            return null;
        }
        Path file = gameDir.resolve(relative).normalize();
        if (!file.startsWith(gameDir) || !Files.isRegularFile(file)) {
            LOGGER.debug("Flint local {} is enabled but its profile file is unavailable", kind);
            return null;
        }
        try {
            NativeImage image = NativeImage.read(Files.newInputStream(file));
            client.getTextureManager().registerTexture(
                    id,
                    new NativeImageBackedTexture(() -> "Flint local " + kind, image)
            );
            LOGGER.debug("Flint local {} texture registered", kind);
            // The one-argument asset constructor converts an ID into a resource-pack path
            // (textures/<id>.png). Local textures are already registered with TextureManager,
            // so both the logical ID and render path must remain the dynamic texture ID.
            return new AssetInfo.TextureAssetInfo(id, id);
        } catch (IOException | RuntimeException error) {
            LOGGER.warn("Could not load Flint local {}: {}", kind, error.getClass().getSimpleName());
            return null;
        }
    }
}
