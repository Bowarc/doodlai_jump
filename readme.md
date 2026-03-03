## Implementation of a simple Doodle jump clone to train a Neural Network on it


### Current state
Ugly test being cleaned up

Display has some issues, namely:  
- Agent is not moving the character
- Ggez seems to have a hard time detecting my main monitor on Wayland/COMSIC DE

### Notes

I went with 540, 960 for the game dimentions

### Project structure

- [**game**](/game/readme.md): Headless implementation of a simple doodle jump clone
- [**display**](/display/readme.md): A window displaying an agent playing the game
- [**ring**](/ring/readme.md): The training ground
