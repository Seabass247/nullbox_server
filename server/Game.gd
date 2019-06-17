extends Spatial

# Declare member variables here. Examples:
# var a = 2
# var b = "text"
var laminar
var players: Dictionary

# Called when the node enters the scene tree for the first time.
func _ready():
	laminar = get_node("/root/Laminar")
	laminar.init_server("12345" as String, self as Node)

func _on_net_player_connected(id: int, data):
	print("New player connected! id= ", id, ", name=", data[1])
	laminar.send_to(id as int, "/root/MainMenu:server_response" as String, ["success"])
	
func _on_net_player_pos(id: int, data):
	print("player_", id, ": pos=", data[0])