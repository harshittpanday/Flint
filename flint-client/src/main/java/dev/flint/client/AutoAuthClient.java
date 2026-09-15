package dev.flint.client;

import com.google.gson.Gson;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.client.network.ServerInfo;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.nio.charset.StandardCharsets;

public final class AutoAuthClient {
    private static final Logger LOGGER = LoggerFactory.getLogger("Flint AutoAuth");
    private static final Gson GSON = new Gson();
    private static final String ENDPOINT = System.getenv("FLINT_AUTOAUTH_ENDPOINT");
    private static final String TOKEN = System.getenv("FLINT_AUTOAUTH_TOKEN");
    private static final AutoAuthStateMachine STATE = new AutoAuthStateMachine();
    private static final int READY_TICKS = 40;
    private static int readyTicks;

    private AutoAuthClient() {
    }

    public static void beginConnection() {
        STATE.beginConnection();
        readyTicks = 0;
        if (ENDPOINT != null && TOKEN != null) {
            LOGGER.debug("AutoAuth: configured; waiting for client readiness");
        }
    }

    public static void tick(MinecraftClient client) {
        if (ENDPOINT == null || TOKEN == null || STATE.state() != AutoAuthStateMachine.State.CONNECTED) {
            return;
        }
        ServerInfo serverInfo = client.getCurrentServerEntry();
        ClientPlayNetworkHandler handler = client.getNetworkHandler();
        if (serverInfo == null || handler == null || client.player == null) {
            readyTicks = 0;
            return;
        }
        if (++readyTicks < READY_TICKS) {
            return;
        }
        String server = serverInfo.address;
        if (!STATE.awaitAuthentication(server)) {
            return;
        }
        STATE.markAttempted();
        LOGGER.debug("AutoAuth: bridge available; authentication requested");
        Thread worker = new Thread(() -> requestCommand(client, handler, server), "Flint-AutoAuth");
        worker.setDaemon(true);
        worker.start();
    }

    private static void requestCommand(
            MinecraftClient client,
            ClientPlayNetworkHandler handler,
            String server
    ) {
        BridgeResponse response = null;
        try {
            int separator = ENDPOINT.lastIndexOf(':');
            if (!ENDPOINT.startsWith("127.0.0.1:") || separator < 0) {
                throw new IllegalStateException("unsafe endpoint");
            }
            int port = Integer.parseInt(ENDPOINT.substring(separator + 1));
            try (Socket socket = new Socket()) {
                socket.connect(new InetSocketAddress("127.0.0.1", port), 1500);
                socket.setSoTimeout(2000);
                BufferedWriter writer = new BufferedWriter(new OutputStreamWriter(socket.getOutputStream(), StandardCharsets.UTF_8));
                writer.write(GSON.toJson(new BridgeRequest(TOKEN, server)));
                writer.newLine();
                writer.flush();
                BufferedReader reader = new BufferedReader(new InputStreamReader(socket.getInputStream(), StandardCharsets.UTF_8));
                response = GSON.fromJson(reader.readLine(), BridgeResponse.class);
            }
        } catch (Exception ignored) {
            LOGGER.debug("AutoAuth: authentication failed; bridge unavailable or rejected the request");
        }
        BridgeResponse result = response;
        client.execute(() -> {
            if (client.getNetworkHandler() == handler
                    && result != null
                    && result.command != null
                    && !result.command.isBlank()) {
                handler.sendChatCommand(result.command);
                STATE.complete();
                LOGGER.debug("AutoAuth: authentication completed");
            } else {
                STATE.complete();
                LOGGER.debug("AutoAuth: authentication failed safely");
            }
        });
    }

    private record BridgeRequest(String token, String server) {
    }

    private static final class BridgeResponse {
        String command;
    }

}
