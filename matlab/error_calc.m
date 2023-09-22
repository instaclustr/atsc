function median_percentage_error = calculate_error(input_signal, output_signal, sampling)
    % Calculate the median percentage error between two time series signals.
    % Input:
    %   input_signal: Input time series signal.
    %   output_signal: Output time series signal.
    %   sampling: Sampling factor for trimming input_signal.
    
    % Input validation
    if ~isnumeric(input_signal) || ~isnumeric(output_signal)
        error('Input and output signals must be numeric arrays.');
    end
    
    if ~isscalar(sampling) || sampling <= 0
        error('Sampling must be a positive scalar.');
    end
    
    % Trim input_signal to match the size of output_signal
    input_trimmed = input_signal(1:sampling:end-1);
    
    % Calculate the percentage error
    scaling_factor = 100; % You can adjust this as needed
    percentage_error = abs(output_signal - input_trimmed) / scaling_factor;
    
    % Calculate and return the median percentage error
    median_percentage_error = median(percentage_error);
end
