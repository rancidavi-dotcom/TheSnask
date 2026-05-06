if exists("b:did_indent")
  finish
endif
let b:did_indent = 1

setlocal autoindent
setlocal indentexpr=SnaskIndent(v:lnum)
setlocal indentkeys=o,O,0},0),0],:,=elif,=else

if exists("*SnaskIndent")
  finish
endif

function! SnaskIndent(lnum) abort
  let l:prevnum = prevnonblank(a:lnum - 1)
  if l:prevnum == 0
    return 0
  endif

  let l:sw = shiftwidth()
  let l:prev = getline(l:prevnum)
  let l:line = getline(a:lnum)
  let l:ind = indent(l:prevnum)

  if l:prev =~# '^\s*\(fun\|class\|if\|elif\|else\|while\|for\|scope\|zone\|with\|unsafe\)\>'
        \ || l:prev =~# '[:{]\s*\($\|//\)'
        \ || l:prev =~# '[@]\s*unsafe\s*$'
        \ || l:prev =~# '[([{]\s*$'
    let l:ind += l:sw
  endif

  if l:line =~# '^\s*\(elif\|else\)\>'
    let l:ind -= l:sw
  endif

  if l:line =~# '^\s*[]})]'
    let l:ind -= l:sw
  endif

  return max([l:ind, 0])
endfunction

