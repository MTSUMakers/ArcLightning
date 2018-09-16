
//test game objects, in the future will be pulling this data from backend
var gameObjects = {
	 game1 : {name:"Touhou", description: "bullet hell with waifus", genre: ["bullet hell", "anime"], thumbnail_path:"images/touhou.jpg", exe_path: "C:\\Users\\THISUSER\\TOUHOU_PATH"},
	 game2 : {name:"Melty Blood", description:"fighter with waifus", genre: ["fighter", "anime", "2d"], thumbnail_path:'images/meltyBlood.png', exe_path: "C:\\Users\\THISUSER\\TOUHOU_PATH"},
	 game3 : {name:"Touhou", description: "bullet hell with waifus", genre: ["bullet hell", "anime"], thumbnail_path:"images/touhou.jpg", exe_path: "C:\\Users\\THISUSER\\TOUHOU_PATH"},
}

//push test objects into array
var gameObjectsArray = [];
for(var key in gameObjects){
	gameObjectsArray.push(gameObjects[key]);
}


//TODO: instead of creating a grid of divs maybe I will create vue components for each of the game cells
function loadGames(){
	var rowNum = 5; //will be number of games/3 
	var colNum = 3;  //set to 3, could be any number of columns
	for(var i = 0; i < rowNum; i++){
		document.write("<div class = 'row' style='margin-right: auto; margin-left: auto;'>");
		for(var x = 0; x < colNum; x++){
			document.write("<div class = 'col-sm-4' style='display:inline-block; height: 33vh; background-color:black; padding:1px;'>");
			//will eventually print out gameObjectsArray[x + (i * 3)]
			document.write("<img style='position:relative; width:100%; height:100%;' src='"  + gameObjectsArray[x].thumbnail_path  + "'/> </div>");
		}
	document.write("</div>");	
	}
}
