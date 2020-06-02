import AutoPilot from "../mod.ts";
var pilot = new AutoPilot();
var screenSize = pilot.screenSize();
var widthUnit: number = screenSize.height / 6;
setInterval(function () {
  var position = pilot.mousePosition();
  if (position.x > widthUnit * 2) {
    pilot.moveMouse(widthUnit, position.y);
  } else {
    pilot.moveMouse(widthUnit * 3, position.y);
  }
}, 200);
