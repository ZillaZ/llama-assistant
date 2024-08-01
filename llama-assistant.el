(require 'json)

(defun transform-message-to-json (message)
  "Transform a string MESSAGE into a specific JSON format."
  (let ((conversation-id "chat 0")
        (role "User"))
    (json-encode `(("conversation_id" . ,conversation-id)
                   ("message" . (("role" . ,role)
                                 ("content" . ,message)))))))

(defun open-new-window-with-buffer (buffer-name)
  "Open a new window and switch to the buffer with BUFFER-NAME."
  (let ((buffer (get-buffer buffer-name)))
    (switch-to-buffer buffer)))

(defun send-message-to-tcp-server (server port message)
  "Llama connection"
  (let* ((buffer (current-buffer))
         (process (open-network-stream "tcp-client" buffer server port)))
    (set-process-sentinel
     process
     (lambda (proc event)
       (when (string= event "connection broken by remote peer\n")
         (with-current-buffer (process-buffer proc)
           (goto-char (point))))))
    (set-process-filter
     process
     (lambda (proc string)
       (with-current-buffer (process-buffer proc)
         (goto-char (point))
         (insert (concat "/*" string "*/")))))
    (process-send-string process message)
    (process-send-eof process)
    buffer))

;; What about this one?
(defun llama-connect (message)
  "Connect to Llama LLM."
  (interactive "sEnter your message: ")
  (let ((json-message (transform-message-to-json message)))
    (setq response-buffer (send-message-to-tcp-server "localhost" 2469 json-message))))

(defun llama-selection (message)
  "Solicits a buffer selection."
  (interactive "sEnter your message: ")
  (if (use-region-p)
      (let ((input (concat message " " (buffer-substring-no-properties (region-beginning) (region-end)))))
    (setq json-message (transform-message-to-json input))
    (send-message-to-tcp-server "localhost" 2469 json-message))
    (error "No region is selected")))
