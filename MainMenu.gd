extends CanvasLayer

var address
var username

# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	

# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
func on_connect():
	address = get_node("Control/ServerAddrBox").text
	username = get_node("Control/UsernameBox").text
	var global = get_node("/root/Global")
	#global.network_uid = OS.get_unique_id()
	global.username = username
	#global.address = "127.0.0.1:12345"
	#print("Connecting to server ", address, " with username ", username, ", network_id ", global.network_uid)
	get_tree().change_scene("res://Game.tscn")
	