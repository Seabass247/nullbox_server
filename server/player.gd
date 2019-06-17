extends KinematicBody

var laminar
var username
var id: int

func _ready():
	laminar = get_node("/root/Laminar")

func _physics_process(delta):
	var pos = get_global_transform().origin
	laminar.send_to(id as int, "/root/Game:set_others_pos" as String, [id, pos])