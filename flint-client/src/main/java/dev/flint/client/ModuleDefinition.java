package dev.flint.client;

public record ModuleDefinition(String id, String name, String description, Category category) {
    public enum Category { HUD, RENDER, PLAYER, FLINT, SETTINGS }
}
