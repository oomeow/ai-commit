# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_ai_commit_global_optspecs
	string join \n a/add h/help V/version
end

function __fish_ai_commit_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_ai_commit_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_ai_commit_using_subcommand
	set -l cmd (__fish_ai_commit_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c ai-commit -n "__fish_ai_commit_needs_command" -s a -l add -d 'Stage all changes before generating the commit message'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -s V -l version -d 'Print version'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -f -a "install" -d 'Install git hooks for AI commit assistance'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -f -a "uninstall" -d 'Remove AI commit hooks'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -f -a "completion" -d 'Generate shell completion script'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -f -a "commit" -d 'Generate AI commit message for staged changes'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -f -a "amend" -d 'Amend the last commit with staged changes using AI-generated message'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -f -a "config" -d 'Manage configuration'
complete -c ai-commit -n "__fish_ai_commit_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand install" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand uninstall" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand completion" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand commit" -l context-limit -d 'Maximum characters to send to AI (default: 200000)' -r
complete -c ai-commit -n "__fish_ai_commit_using_subcommand commit" -l output-file -d 'Write generated message to file' -r -F
complete -c ai-commit -n "__fish_ai_commit_using_subcommand commit" -s a -l add -d 'Stage all changes before generating the commit message'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand commit" -l dry-run -d 'Show generated message without committing'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand commit" -l generate-only -d 'Generate commit message only (no commit, no confirmation)'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand commit" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand amend" -l context-limit -d 'Maximum characters to send to AI (default: 200000)' -r
complete -c ai-commit -n "__fish_ai_commit_using_subcommand amend" -l dry-run -d 'Show generated message without amending'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand amend" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and not __fish_seen_subcommand_from show init edit help" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and not __fish_seen_subcommand_from show init edit help" -f -a "show" -d 'Show current configuration'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and not __fish_seen_subcommand_from show init edit help" -f -a "init" -d 'Initialize default configuration'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and not __fish_seen_subcommand_from show init edit help" -f -a "edit" -d 'Edit configuration in your terminal editor'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and not __fish_seen_subcommand_from show init edit help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and __fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show current configuration'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "init" -d 'Initialize default configuration'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit configuration in your terminal editor'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and not __fish_seen_subcommand_from install uninstall completion commit amend config help" -f -a "install" -d 'Install git hooks for AI commit assistance'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and not __fish_seen_subcommand_from install uninstall completion commit amend config help" -f -a "uninstall" -d 'Remove AI commit hooks'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and not __fish_seen_subcommand_from install uninstall completion commit amend config help" -f -a "completion" -d 'Generate shell completion script'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and not __fish_seen_subcommand_from install uninstall completion commit amend config help" -f -a "commit" -d 'Generate AI commit message for staged changes'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and not __fish_seen_subcommand_from install uninstall completion commit amend config help" -f -a "amend" -d 'Amend the last commit with staged changes using AI-generated message'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and not __fish_seen_subcommand_from install uninstall completion commit amend config help" -f -a "config" -d 'Manage configuration'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and not __fish_seen_subcommand_from install uninstall completion commit amend config help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "show" -d 'Show current configuration'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "init" -d 'Initialize default configuration'
complete -c ai-commit -n "__fish_ai_commit_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "edit" -d 'Edit configuration in your terminal editor'
