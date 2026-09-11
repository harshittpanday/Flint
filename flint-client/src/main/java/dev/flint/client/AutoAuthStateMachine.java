package dev.flint.client;

final class AutoAuthStateMachine {
    enum State {
        DISCONNECTED,
        CONNECTED,
        AWAITING_AUTHENTICATION,
        AUTHENTICATION_ATTEMPTED,
        SESSION_COMPLETE
    }

    private State state = State.DISCONNECTED;
    private String connectedServer;

    void beginConnection() {
        connectedServer = null;
        state = State.CONNECTED;
    }

    void disconnect() {
        connectedServer = null;
        state = State.DISCONNECTED;
    }

    boolean awaitAuthentication(String server) {
        if (!server.equalsIgnoreCase(connectedServer)) {
            connectedServer = server;
            state = State.CONNECTED;
        }
        if (state == State.AUTHENTICATION_ATTEMPTED || state == State.SESSION_COMPLETE) {
            return false;
        }
        state = State.AWAITING_AUTHENTICATION;
        return true;
    }

    void markAttempted() {
        if (state == State.AWAITING_AUTHENTICATION) {
            state = State.AUTHENTICATION_ATTEMPTED;
        }
    }

    void complete() {
        state = State.SESSION_COMPLETE;
    }

    State state() {
        return state;
    }
}
