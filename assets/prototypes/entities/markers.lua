define {
    "entity",
    id = "base_marker",
    abstract = true,
    mesh = "models/bad_marker.glb",
    components = {
        "Marker",
    }
}

define {
    "entity",
    id = "spawner_human",
    parent = "base_marker"
}

define {
    "entity",
    id = "spawner_observer",
    parent = "base_marker"
}

