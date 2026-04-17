-- showcase prototypes

define {
    "entity",
    id = "task",
    mesh = "models/task.glb",
    components = {
        "SimpleTask",
        "Interactable",
    }
}

define {
    "entity",
    id = "spawner_task",
    parent = "base_marker",
    components = {
        "TaskSpawner"
    }
}

