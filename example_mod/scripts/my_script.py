import sims4.commands


@sims4.commands.Command("helloworld", command_type=sims4.commands.CommandType.Live)
def hello_world(_connection=None):
    """
    A simple command that prints "Hello, World!" to the game's console.
    """
    output = sims4.commands.CheatOutput(_connection)
    output("Hello, World!")
