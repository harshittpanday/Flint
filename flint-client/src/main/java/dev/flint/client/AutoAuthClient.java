package dev.flint.client;

import com.google.gson.Gson;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.client.network.ServerInfo;
import net.minecraft.text.Text;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.nio.charset.StandardCharsets;

public final class AutoAuthClient {
    private static final Gson GSON = new Gson();
    private static final String ENDPOINT = System.getenv("FLINT_AUTOAUTH_ENDPOINT");
    private static final String TOKEN = System.getenv("FLINT_AUTOAUTH_TOKEN");
    private static final AutoAuthStateMachine STATE = new AutoAuthStateMachine();

    private AutoAuthClient() {
    }

    public static void beginConnection() {
        STATE.beginConnection();
    }

    public static boolean intercept(String command, ClientPlayNetworkHandler handler) {
        if (!command.equals("flintauth login") && !command.equals("flintauth register")) {
            return false;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        ServerInfo serverInfo = client.getCurrentServerEntry();
        if (serverInfo == null || ENDPOINT == null || TOKEN == null) {
            notifyPlayer(client, "AutoAuth is not configured for this session.");
            return true;
        }
        String server = serverInfo.address;
        if (!STATE.awaitAuthentication(server)) {
            notifyPlayer(client, "AutoAuth already attempted for this connection.");
            return true;
        }
        String action = command.endsWith("register") ? "register" : "login";
        STATE.markAttempted();
        Thread worker = new Thread(() -> requestCommand(client, handler, server, action), "Flint-AutoAuth");
        worker.setDaemon(true);
        worker.start();
        return true;
    }

    private static void requestCommand(
            MinecraftClient client,
            ClientPlayNetworkHandler handler,
            String server,
            String action
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
                writer.write(GSON.toJson(new BridgeRequest(TOKEN, server, action)));
                writer.newLine();
                writer.flush();
                BufferedReader reader = new BufferedReader(new InputStreamReader(socket.getInputStream(), StandardCharsets.UTF_8));
                response = GSON.fromJson(reader.readLine(), BridgeResponse.class);
            }
        } catch (Exception ignored) {
            // Details could contain private local state, so only a generic in-game status is shown.
        }
        BridgeResponse result = response;
        client.execute(() -> {
            if (result != null && result.command != null && !result.command.isBlank()) {
                handler.sendChatCommand(result.command);
                STATE.complete();
                notifyPlayer(client, "AutoAuth command sent once.");
            } else {
                STATE.complete();
                notifyPlayer(client, "AutoAuth could not provide a command for this server.");
            }
        });
    }

    private static void notifyPlayer(MinecraftClient client, String message) {
        if (client.player != null) {
            client.player.sendMessage(Text.literal("[Flint] " + message), false);
        }
    }

    private record BridgeRequest(String token, String server, String action) {
    }

    private static final class BridgeResponse {
        String command;
    }

}
