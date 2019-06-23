extends KinematicBody

var laminar
var username
var id: int

func _ready():
	laminar = get_node("/root/Laminar")

func _physics_process(delta):
	var pos = get_global_transform().origin
	laminar.send_to(0 as int, "/root/Game:set_others_pos" as String, [id, pos as Vector3])
	laminar.send_to(id as int, "/root/Game:pos_relayed" as String, [id, pos as Vector3])
	
func move_to_pos(pos: Vector3):
	global_transform.origin = pos
	print("Moved player_", id," to ", pos)