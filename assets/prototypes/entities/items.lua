define {
    "entity",
    id = "base_item",
    abstract = true,
    components = {
        "Item",
        "RigidBody",
        "ItemCollider",
        Mass = 10,
        "Interactable",
        Weapon = {
            damage = 20,
            hit_sound = "sounds/hit1.ogg",
        },
        PlaySoundOnPickup = {
            sound = "sounds/component_pickup.ogg"
        },
        PlaySoundOnDrop = {
            sound = "sounds/component_drop.ogg"
        }
    },
}

define {
    "entity",
    id = "light",
    parent = "base_item",
    mesh = "models/item.glb",
    components = {
        "Light",
    }
}

define {
    "entity",
    id = "iron",
    parent = "base_item",
    mesh = "models/item.glb",
    components = {
        "Stack",
    }
}

define {
    "entity",
    id = "bike_horn",
    parent = "base_item",
    mesh = "models/ss3d/bikehorn.glb",
    components = {
        "Item",
        "RigidBody",
        PlaySoundOnUse = {
            sound = "sounds/bikehorn.ogg"
        }
    }
}
