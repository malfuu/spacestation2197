define({
    "entity",
    id = "base_structure",
    abstract = true,
    components = {},
})

define({
    "entity",
    id = "vent",
    mesh = "models/ss3d/vent.glb",
    components = {
        "Vent",
    },
})

define({
    "entity",
    id = "scrubber",
    mesh = "models/ss3d/scrubber.glb",
    components = {
        "Scrubber",
    },
})
