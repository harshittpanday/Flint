package dev.flint.client;

public final class AutoAuthStateMachineTest {
    public static void main(String[] args) {
        sameServerReconnectResetsCompletedSession();
        differentServerStartsAFreshAttempt();
        disconnectAndAllTransitionsAreExplicit();
        loginAndRegisterAreOneShotPerConnection();
    }

    private static void sameServerReconnectResetsCompletedSession() {
        AutoAuthStateMachine state = completed("play.example.net");
        require(!state.awaitAuthentication("play.example.net"), "same session must reject repeated auth");
        state.beginConnection();
        require(state.state() == AutoAuthStateMachine.State.CONNECTED, "same-server reconnect must reset");
        require(state.awaitAuthentication("play.example.net"), "same-server next session must allow one auth");
    }

    private static void differentServerStartsAFreshAttempt() {
        AutoAuthStateMachine state = completed("one.example.net");
        require(state.awaitAuthentication("two.example.net"), "different server must start a fresh session");
        require(state.state() == AutoAuthStateMachine.State.AWAITING_AUTHENTICATION, "new server must await auth");
    }

    private static void disconnectAndAllTransitionsAreExplicit() {
        AutoAuthStateMachine state = new AutoAuthStateMachine();
        require(state.state() == AutoAuthStateMachine.State.DISCONNECTED, "initial state");
        state.beginConnection();
        require(state.state() == AutoAuthStateMachine.State.CONNECTED, "connected state");
        require(state.awaitAuthentication("play.example.net"), "await trigger");
        require(state.state() == AutoAuthStateMachine.State.AWAITING_AUTHENTICATION, "awaiting state");
        state.markAttempted();
        require(state.state() == AutoAuthStateMachine.State.AUTHENTICATION_ATTEMPTED, "attempted state");
        state.complete();
        require(state.state() == AutoAuthStateMachine.State.SESSION_COMPLETE, "complete state");
        state.disconnect();
        require(state.state() == AutoAuthStateMachine.State.DISCONNECTED, "disconnected state");
    }

    private static void loginAndRegisterAreOneShotPerConnection() {
        AutoAuthStateMachine login = completed("login.example.net");
        require(!login.awaitAuthentication("login.example.net"), "login must not repeat");
        AutoAuthStateMachine register = completed("register.example.net");
        require(!register.awaitAuthentication("register.example.net"), "register must not repeat");
    }

    private static AutoAuthStateMachine completed(String server) {
        AutoAuthStateMachine state = new AutoAuthStateMachine();
        state.beginConnection();
        require(state.awaitAuthentication(server), "first auth must be allowed");
        state.markAttempted();
        state.complete();
        return state;
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }
}
