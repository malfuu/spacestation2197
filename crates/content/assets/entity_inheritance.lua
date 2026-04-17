define {
    "entity",
    id = "base_template",
    mesh = "template.glb",
    abstract = true,
    components = {
        "CompA",
        "CompB"
    }
}

define {
    "entity",
    id = "child_1",
    parent = "base_template",
    components = {
        CompA = { val_1 = 50.0, val_2 = 100.0 },
    }
}

define {
    "entity",
    id = "child_2",
    parent = "base_template",
    components = {
        --
    }
}
