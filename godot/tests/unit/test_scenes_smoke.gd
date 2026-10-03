## Instantiates every gameplay scene for a couple of frames. Catches parse
## errors and broken node paths that pure-logic tests can't see.
extends TestSuite

const SCENE_DIRS := ["res://scenes/actors", "res://scenes/ui", "res://scenes/world"]

func _scene_paths() -> PackedStringArray:
	var out := PackedStringArray()
	for dir_path in SCENE_DIRS:
		for file in DirAccess.get_files_at(dir_path):
			if file.ends_with(".tscn"):
				out.push_back("%s/%s" % [dir_path, file])
	return out

func test_every_scene_loads_and_instantiates() -> void:
	for path in _scene_paths():
		var packed: PackedScene = load(path)
		assert_true(packed != null, "failed to load %s" % path)
		if packed == null:
			continue
		var node := packed.instantiate()
		assert_true(node != null, "failed to instantiate %s" % path)
		add_child(node)
		node.queue_free()
