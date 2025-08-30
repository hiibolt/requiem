# Requiem
**An AI-Powered Visual Novel Game Engine**

Requiem is a cutting-edge visual novel engine built with Rust and Bevy, featuring revolutionary AI-driven character interactions powered by OpenAI's GPT models. Create dynamic, responsive visual novels where characters have goals, memories, and adaptive conversations with players.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/bevy-2C2D33?style=for-the-badge&logo=bevy&logoColor=white)
![OpenAI](https://img.shields.io/badge/OpenAI-412991?style=for-the-badge&logo=openai&logoColor=white)

## ✨ Features

### 🎭 **Advanced Character System**
- **Dynamic Character Management**: JSON-based character definitions with customizable attributes
- **Emotion System**: Real-time emotion changes that affect character sprites and dialogue
- **Multi-Outfit Support**: Characters can switch between different outfits and emotional states
- **Character Descriptions**: Rich personality profiles that inform AI behavior

### 🤖 **AI-Powered Interactions**
- **Goal-Oriented AI**: Characters pursue specific objectives during conversations
- **Dynamic Dialogue Generation**: Real-time conversation generation using GPT-3.5-turbo
- **Intelligent Goal Assessment**: AI determines when character objectives are achieved
- **Context-Aware Responses**: Characters remember conversation history and context
- **Player Input Processing**: Natural language input system for authentic interactions

### 🎨 **Rich Visual Experience**
- **Dynamic Backgrounds**: Environment changes based on story progression
- **Character Sprites**: Emotion-based sprite switching with fade transitions
- **Custom GUI System**: Modular interface with themed textboxes and UI elements
- **Typing Animation**: Smooth text scrolling effects for immersive reading

### 📝 **Flexible Scripting Engine**
- **Custom Script Language**: Bash-like syntax for easy story creation
- **Scene Management**: Seamless transitions between story segments
- **Command System**: Rich set of commands for controlling game flow
- **Event-Driven Architecture**: Responsive system for handling user interactions

### 🔧 **Developer-Friendly**
- **Modular Plugin System**: Built on Bevy's ECS architecture
- **Hot-Reloadable Assets**: Dynamic loading of scripts, sprites, and configurations
- **Cross-Platform**: Runs on Windows, macOS, and Linux
- **Nix Integration**: Reproducible development environment with flake.nix

## 🚀 Quick Start

### Prerequisites
- Rust (latest stable)
- OpenAI API key
- Git

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/requiem.git
   cd requiem
   ```

2. **Set up your OpenAI API key:**
   ```bash
   export OPENAI_API_KEY=your_api_key_here
   ```

3. **Run the game:**
   ```bash
   cargo run
   ```

### Using Nix (Recommended)
```bash
nix develop  # Enter development shell
cargo run    # Build and run
```

## 📚 Script Language Reference

Requiem uses a custom scripting language with bash-like syntax for defining story flow:

### Basic Commands

```bash
# Character dialogue
say character=`Nayu` msg=`Hello, how are you today?`

# Player dialogue
psay msg=`I'm doing great, thanks for asking!`

# Set character emotion
set type=`emotion` character=`Nayu` emotion=`HAPPY`

# Change background
set type=`background` background=`main_classroom_day`

# AI-powered dialogue with goals
gpt character=`Nayu` goal=`Find out what the player is interested in`

# Scene transitions
scene id=`scene2`

# Logging (development)
log msg=`Debug message here`
```

### Advanced Features

```bash
# GUI customization
set type=`GUI` id=`_textbox_background` sprite=`TEXTBOX_NAYU`

# Player input collection
# This is triggered automatically when AI needs player response

# End scene
end
```

## 🎨 Asset Organization

```
assets/
├── backgrounds/          # Background images
│   ├── main_classroom_day.png
│   └── main_classroom_night.png
├── characters/           # Character data and sprites
│   └── CharacterName/
│       ├── character.json
│       └── outfit_name/
│           ├── HAPPY.png
│           ├── SAD.png
│           └── NEUTRAL.png
├── gui/                  # UI elements
│   ├── TEXTBOX_CHARACTER.png
│   └── NAMEBOX.png
├── fonts/               # Font files
└── scripts/             # Story scripts
    ├── entry.txt        # Entry point (required)
    └── scene1.txt
```

### Character Configuration

Characters are defined in JSON format:

```json
{
    "name": "Nayu",
    "age": "18",
    "default_outfit": "uniform_neutral",
    "default_emotion": "NEUTRAL",
    "emotions": ["HAPPY", "SAD", "NEUTRAL", "CONCERNED"],
    "description": "A thoughtful student who enjoys music and programming...",
    "hobbies": "music, guitar, drawing, cooking"
}
```

## 🤖 AI System Deep Dive

### How AI Characters Work

1. **Goal Assignment**: Each AI interaction has a specific goal (e.g., "Learn about the player's interests")
2. **Context Building**: The AI receives character description, conversation history, and current goal
3. **Response Generation**: GPT generates contextually appropriate dialogue
4. **Goal Assessment**: A separate AI call determines if the character achieved their objective
5. **Adaptive Flow**: Based on goal completion, the story can branch or continue

### AI Integration Features

- **Conversation Memory**: Full conversation history is maintained and provided to AI
- **Character Consistency**: AI responses stay true to character descriptions and personalities
- **Error Handling**: Robust fallback system for API failures or network issues
- **Token Usage Tracking**: Monitor API usage for cost management

## 🏗️ Architecture

Requiem is built on Bevy's Entity Component System (ECS) with distinct modules:

- **Compiler Module**: Parses script files and converts them to executable transitions
- **Character Module**: Manages character sprites, emotions, and properties
- **Chat Module**: Handles dialogue display, player input, and text animation
- **Intelligence Module**: Interfaces with OpenAI API for dynamic conversations
- **Background Module**: Controls scene backgrounds and environmental changes

## 🔧 Configuration

### Environment Variables
- `OPENAI_API_KEY`: Your OpenAI API key (required)

### Game Settings
Player name and other settings are currently configured in `src/main.rs`:

```rust
game_state.playername = String::from("YourName");
```

## 🤝 Contributing

We welcome contributions! Here are some areas where you can help:

- **UI/UX Improvements**: Enhanced text input, visual effects
- **Script Language Features**: New commands and functionality  
- **Performance Optimization**: Better asset loading and memory management
- **AI Enhancements**: Improved goal systems and character behavior
- **Cross-Platform Support**: Testing and fixes for different platforms

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📋 Roadmap

### Completed ✅
- [x] Character system with emotions and outfits
- [x] AI-powered dialogue generation
- [x] Goal-oriented character behavior
- [x] Custom scripting language
- [x] Scene management
- [x] Dynamic backgrounds
- [x] Player input system

### In Progress 🚧
- [ ] Enhanced text input system
- [ ] Visual transition effects
- [ ] Save/load system
- [ ] Audio integration

### Planned 📅
- [ ] Visual script editor
- [ ] Multiplayer support
- [ ] Mobile platform support
- [ ] Steam Workshop integration
- [ ] Advanced AI personality system

## 🙏 Acknowledgments

- Built with [Bevy Engine](https://bevyengine.org/)
- AI powered by [OpenAI](https://openai.com/)
- Development environment managed with [Nix](https://nixos.org/)
- Special thanks to the Rust and game development communities