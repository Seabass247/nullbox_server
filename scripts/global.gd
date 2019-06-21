extends Node

var laminar

var username
var client_id: int
var network_heartbeat

func _ready():
    laminar = get_node("/root/Laminar")
