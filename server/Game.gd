extends Spatial

# Declare member variables here. Examples:
# var a = 2
# var b = "text"
var laminar

# Called when the node enters the scene tree for the first time.
func _ready():
	laminar = get_node("/root/Laminar")
	laminar.init_server("12345" as String, self as Node)

func _on_net_player_connected(data):
	print("New player connected: ", data[1])
	