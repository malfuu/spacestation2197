define {
    "TypeB",
    id = "base_abstract",
    abstract = true,
    val_1 = 10,
    val_2 = 100,
    flag = false,
}

define {
    "TypeB",
    id = "child_level_1",
    parent = "base_abstract",
    val_1 = 15,
}

define {
    "TypeB",
    id = "child_level_2",
    parent = "child_level_1",
    flag = true,
}
