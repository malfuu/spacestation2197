-- lua not allowing bit operations on rust
-- should be concerning already
local LAYERS = {
    NONE = 0,
    NORMAL = 1,
    GHOST = 64,
    GHOST_BLOCKING = 128,
    ALL = 4294967295,
}

local function make_collision_layers(memberships, filters)
    return {
        memberships = memberships,
        filters = filters,
    }
end

define({
    "entity",
    id = "base_mob",
    abstract = true,
    components = {
        "Mob",
        "Interactable",
        Mass = 70.0,
    },
})

define({
    "entity",
    id = "moving_mob",
    abstract = true,
    parent = "base_mob",
    components = {
        CollisionLayers = make_collision_layers(LAYERS.NORMAL, LAYERS.NORMAL),
        "MobCollider",
        "Listener",
        "Speaker",

        "RigidBody",
        "MobController",
    },
})

define({
    "entity",
    id = "ghost",
    parent = "moving_mob",
    mesh = "models/ss3d/ghost.glb",
    components = {
        CollisionLayers = make_collision_layers(LAYERS.GHOST, LAYERS.GHOST_BLOCKING),
        "Ghost",
    },
})

define({
    "entity",
    id = "admin_ghost", -- ghost with hands!
    parent = "ghost",
    mesh = "models/ss3d/ghost.glb",
    components = {
        CollisionLayers = make_collision_layers(LAYERS.GHOST, LAYERS.GHOST_BLOCKING),
        "Ghost",
        "Hands",
    },
})

define({
    "entity",
    id = "human",
    parent = "moving_mob",
    mesh = "models/ss3d/human.glb",
    components = {
        "Hands",
        "InteractCooldown",
        Health = { amount = 100 },
    },
})
