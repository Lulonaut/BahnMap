import * as d3 from "https://cdn.jsdelivr.net/npm/d3@7/+esm";

const map = L.map("map")
  .setView([51.16, 10.45], 1)
  .fitBounds([
    [54.163046, 6.878084],
    [48.97136, 15.209999],
  ]);

L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {}).addTo(map);

fetch("result.json")
  .then((response) => response.json())
  .then((json) => parseJson(json));

function parseJson(jsonData) {
  const interpolator = d3.interpolateRgb("rgb(157,201,236)", "rgb(129,15,15)");

  let biggestWeight = 0;
  let biggestRadius = 15000;
  for (let i = 0; i < jsonData.length; i++) {
    let element = jsonData[i];
    if (element["weight"] > biggestWeight) {
      biggestWeight = element["weight"];
    }
  }
  console.log(biggestWeight);

  for (let i = 0; i < jsonData.length; i++) {
    let element = jsonData[i];
    let ratio = element["weight"] / biggestWeight;

    L.circle([element["lat"], element["long"]], {
      color: interpolator(element["result"]),
      fillColor: interpolator(element["result"]),
      fillOpacity: 0.5,
      radius: biggestRadius * ratio,
    }).addTo(map);
  }
}
