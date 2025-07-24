# kilo-CLI



A **Rust-powered command-line tool** for interacting with the [Kilonova](https://kilonova.ro/) API.  
Effortlessly manage your competitive programming workflow — login, view problems, submit solutions, and more!

---

## Demo 👀


https://github.com/user-attachments/assets/57755eda-3440-4c07-bd86-75785c176437


## ✨ Features

- **Authentication**
    - Secure login and logout, with session token management.
    - Extend your session for uninterrupted productivity.

- **User Info**
    - Retrieve your user data (`me`).

- **Problem Interaction**
    - Search for problems by name.
    - View the last seen problem statement directly in your terminal.
    - Set preferred language for problem statements and submissions.

- **Submission Workflow**
    - Submit solutions to the last viewed problem.
    - Real-time judging feedback, including verdicts (correct, wrong answer, TLE, MLE, etc).

- **Robust Terminal Experience**
    - Color-coded output for verdicts and notifications.
    - Interactive paginated problem statements.

---

## 🚀 Getting Started

### Prerequisites

- Internet access (to communicate with the Kilonova API)

### Installation

Download the pre-built binary for your operating system from the [Releases](https://github.com/RaduTeodorProsie/kilo-CLI/releases) section.

No need to clone or build the project yourself.

### Usage

To view all available commands, run:

```bash
./kilo-CLI --help
```

---

## 🔒 Credentials & Security

kilo-CLI uses your system keyring for securely storing tokens, session data, and preferences. No extra configuration files. 
If you are using linux, you might need to create a keyring named "login" (the default name) if you haven't done so already. 
---

## 📄 License

This project is licensed under the Creative Commons Attribution 4.0 International (CC BY-SA 4.0) License.
See [this link](https://creativecommons.org/licenses/by-sa/4.0/) for details.

---

> _kilo-CLI is not affiliated with Kilonova platform. This is an unofficial client tool._
