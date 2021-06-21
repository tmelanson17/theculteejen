#include <VariableTimedAction.h>
#include <SoftwareSerial.h>

#include "FastLED.h"
#define NUM_LEDS 50
CRGB leds[NUM_LEDS];
CRGB ROULETTE[] = {CRGB::Beige, CRGB::Blue, CRGB::Red, CRGB::Yellow, CRGB::Green, CRGB::DarkCyan, CRGB::Magenta, CRGB::DarkBlue, CRGB::Maroon, CRGB::MediumPurple};
int startInteger = 0;
size_t roulette_length = sizeof(ROULETTE)/sizeof(ROULETTE[0])-1;
CRGB color;
bool dim = false;

unsigned char incomingBytes[4];
unsigned char storePacket[10];
const char HEADER[] = {'a', 'z'};
unsigned long duration = 1000;
unsigned long msBeatTime = 0;
size_t colorIndex = 0;


// Non-class strobe function
size_t change_color() {
  return random(0, roulette_length);
}
 
void strobe(const unsigned long currentTime, const unsigned long lastBeatMs, const unsigned long duration, size_t color) {
    CRGB output = ROULETTE[color];
    double fraction_of_strobe = double(currentTime - lastBeatMs) / duration;
    uint8_t fade = min(int(fraction_of_strobe*256), 255); // Amount to fade, maximum being total darkness.
    output.fadeToBlackBy(fade);
    for (int i = 0; i<NUM_LEDS; ++i) {
      leds[i] = output;
      leds[i].fadeToBlackBy(120);
    }
}


class SerialEvent : public VariableTimedAction {
  public:


  //this method will be called at your specified interval
  unsigned long run() override {
    unsigned long currentTime = millis();
    if (Serial.available() >= 8) {
          String output = Serial.readString();
//        Serial.println(output);
//        Serial.println("Other stuff");
//        Serial.println(HEADER);

        if (output[0] == HEADER[0] && output[1] == HEADER[1]) {
    
          unsigned char bit1 = output[output.length() - 6];
          unsigned char bit2 = output[output.length() - 5];
          unsigned char bit3 = output[output.length() - 4];
          unsigned char bit4 = output[output.length() - 3];
          unsigned long msIntoBeat = (unsigned long)(bit4)*256 + (unsigned long)(bit3);
          unsigned long msToNextBeat = (unsigned long)(bit2)*256 + (unsigned long)(bit1);
      
          duration = msToNextBeat + msIntoBeat;
          msBeatTime = max(currentTime, msIntoBeat)  - msIntoBeat;
    //      Serial.println(currentTime);
    //      Serial.println(msBeatTime);
    //    }
//        }
      }
      
      unsigned char acceptByte[] = {(unsigned char)(duration)};
      Serial.print(output);
  } 
  }
  unsigned long lastUpdate = 0;
};

  
class LED : public VariableTimedAction {
public:

  //this method will be called at your specified interval
  unsigned long run() override {
    unsigned long currentTime = millis();  
    // Change color if next beat.
    while ((currentTime - msBeatTime)  > duration) {
      colorIndex = change_color();
      msBeatTime += duration;
    }
  
    strobe(currentTime, msBeatTime, duration, colorIndex);
  
    FastLED.show();
  }
  unsigned long lastUpdate = 0;

};

LED led;
SerialEvent serial;;
void setup() { 
  FastLED.addLeds<NEOPIXEL, 6>(leds, NUM_LEDS); 
  Serial.begin(115200);
}

bool equal(char arr1[], char arr2[], int len) {
  for (size_t i = 0; i < len; ++i) {
    if (arr1[i] != arr2[i]) {
      return false;
    }
  }
  return true;
}


void loop() {  
  if (millis() - serial.lastUpdate > 10000) {
    serial.run();
    serial.lastUpdate = millis();
  }
  if (millis() - led.lastUpdate > 20) {
    led.run();
    led.lastUpdate = millis();
  }
}
