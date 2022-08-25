local null_ls = require("null-ls")
local null_ls_helpers = require("null-ls.helpers")
local null_ls_methods = require("null-ls.methods")

local CODE_ACTION = null_ls_methods.internal.CODE_ACTION
local DIAGNOSTICS = null_ls_methods.internal.DIAGNOSTICS

local handle_ltcc_codeaction_output = function(params)
	local actions = {}

	for _, m in ipairs(params.output) do
		if m.replacements ~= vim.NIL and params.row == m.moreContext.line_number then
			local row = m.moreContext.line_number - 1
			local col_beg = m.moreContext.line_offset
			local col_end = m.moreContext.line_offset + m.length

			for _, r in ipairs(m.replacements) do
				if string.find(r.value, "not shown") == nil then
					table.insert(actions, {
						title = "Replace with “" .. r.value .. "”",
						action = function()
							vim.api.nvim_buf_set_text(params.bufnr, row, col_beg, row, col_end, { r.value })
						end,
					})
				end
			end
		end
	end

	return actions
end

local handle_ltcc_diag_output = function(params)
	local file = params.output
	if file and file then
		local parser = null_ls_helpers.diagnostics.from_json({
			severities = {
				ERROR = null_ls_helpers.diagnostics.severities.error,
			},
		})

		local offenses = {}

		for _, m in ipairs(file) do
			local tip = table.concat(
				vim.tbl_map(function(r)
					return "“" .. r.value .. "”"
				end, m.replacements),
				", "
			)

			table.insert(offenses, {
				message = m.message .. " Try: " .. tip,
				ruleId = m.rule.id,
				level = "ERROR",
				line = m.moreContext.line_number,
				column = m.moreContext.line_offset + 1,
				endLine = m.moreContext.line_number,
				endColumn = m.moreContext.line_offset + m.length + 1,
			})
		end

		return parser({ output = offenses })
	end

	return {}
end

local lctt_filetypes = {
	"bash",
	"css",
	"dockerfile",
	"go",
	"html",
	"javascript",
	"lua",
	"make",
	"python",
	"rust",
	"sql",
	"terraform",
	"toml",
	"typescript",
}

local ltcc_code_action = null_ls_helpers.make_builtin({
	name = "ltcc",
	meta = {
		url = "https://github.com/dustinblackman/languagetool-code-comments",
		description = "languagetool-code-comments integrates the LanguageTool API to parse, spell check, and correct the grammar of your code comments!",
	},
	method = CODE_ACTION,
	filetypes = lctt_filetypes,
	generator_opts = {
		command = "languagetool-code-comments",
		args = { "check", "-l", "en-US", "-f", "$FILENAME" },
		format = "json",
		timeout = 60000,
		check_exit_code = function(c)
			return c <= 1
		end,
		on_output = handle_ltcc_codeaction_output,
		use_cache = false,
	},
	factory = null_ls_helpers.generator_factory,
})

local ltcc_diag = null_ls_helpers.make_builtin({
	name = "ltcc",
	meta = {
		url = "https://github.com/dustinblackman/languagetool-code-comments",
		description = "languagetool-code-comments integrates the LanguageTool API to parse, spell check, and correct the grammar of your code comments!",
	},
	method = DIAGNOSTICS,
	filetypes = lctt_filetypes,
	generator_opts = {
		command = "languagetool-code-comments",
		args = { "check", "-l", "en-US", "-f", "$FILENAME" },
		format = "json",
		timeout = 60000,
		check_exit_code = function(c)
			return c <= 1
		end,
		on_output = handle_ltcc_diag_output,
		use_cache = false,
	},
	factory = null_ls_helpers.generator_factory,
})

null_ls.setup({
	sources = {
		ltcc_code_action,
		ltcc_diag,
	},
})
